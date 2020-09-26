use crate::serde_structs::structs::{Instr, Literal, Var, Op};
use crate::serde_structs::namer;
use std::collections::HashMap;
use std::ops::RangeInclusive;

type LNum = usize;
#[derive(Debug, Clone)]
enum Value {
    Unknown,
    Literal(Literal),
    UnaryOp(Op, LNum),
    BinaryOp(Op, LNum, LNum),
    Call(Vec<LNum>)
}

impl Value {
    fn canonicalize(&mut self) {
        if let Value::BinaryOp(op, v1, v2) = self {
            if op.is_commutative() && v1 > v2{
                    std::mem::swap(v1, v2);
            }
        }
    }

    fn transform_special_form(&mut self, tbl: &Table) {
        match self {
            Value::UnaryOp(op, arg) => {
                if let Op::Id = op { // Unary Id on const
                   if let lit@Value::Literal(..) = tbl.get_row_value(*arg) {
                        *self = lit.clone();
                   } else if let Value::UnaryOp(Op::Id, row) = tbl.get_row_value(*arg) {
                        *self = Value::UnaryOp(Op::Id, *row);
                    }
                } else if let Op::Not = op { // Unary not on const
                    if let Value::Literal(ref inner) = tbl.get_row_value(*arg) {
                        *self = Value::Literal(!inner.clone())
                   }
                }
            }
            Value::BinaryOp(op, a1, a2) => {
                // Computation that we can perform at compile time
                if let (Value::Literal(ref l1),
                        Value::Literal(ref l2)) = (tbl.get_row_value(*a1), tbl.get_row_value(*a2)) {
                            match op {
                                Op::Add => {
                                    *self = Value::Literal(l1.clone() + l2.clone())
                                }
                                Op::Sub => {
                                    *self = Value::Literal(l1.clone() - l2.clone())
                                }
                                Op::Mul => {
                                    *self = Value::Literal(l1.clone() * l2.clone())
                                }
                                Op::Div => {
                                    *self = Value::Literal(l1.clone() / l2.clone())
                                }
                                Op::Eq => {
                                    if let (Literal::Int(ref i1),
                                       Literal::Int(ref i2)) = (l1, l2){
                                        *self = Value::Literal(Literal::Bool(i1 == i2))
                                       }
                                }
                                Op::Lt => {
                                    if let (Literal::Int(ref i1),
                                       Literal::Int(ref i2)) = (l1, l2){
                                        *self = Value::Literal(Literal::Bool(i1 < i2))
                                       }
                                }
                                Op::Gt => {
                                    if let (Literal::Int(ref i1),
                                       Literal::Int(ref i2)) = (l1, l2){
                                        *self = Value::Literal(Literal::Bool(i1 > i2))
                                       }
                                }
                                Op::Le => {
                                    if let (Literal::Int(ref i1),
                                       Literal::Int(ref i2)) = (l1, l2){
                                        *self = Value::Literal(Literal::Bool(i1 <= i2))
                                       }
                                }
                                Op::Ge => {
                                    if let (Literal::Int(ref i1),
                                       Literal::Int(ref i2)) = (l1, l2){
                                        *self = Value::Literal(Literal::Bool(i1 >= i2))
                                       }
                                }
                                Op::And => {
                                    *self = Value::Literal(l1.clone() & l2.clone())
                                }
                                Op::Or => {
                                    *self = Value::Literal(l1.clone() | l2.clone())
                                }
                                _ => {}
                            }
                        }
                // Special
                else if let (_, Value::Literal(ref l))
                           | (Value::Literal(ref l), _)
                           = (tbl.get_row_value(*a1), tbl.get_row_value(*a2)) {
                    let unknown_row = if let Value::Literal(..) = tbl.get_row_value(*a1) {
                        a2 } else { a1 };
                    match (op, l) {
                        (Op::Add, Literal::Int(0)) | (Op::Mul, Literal::Int(1))
                        | (Op::Or, Literal::Bool(false)) | (Op::And, Literal::Bool(true))=> {
                            *self = Value::UnaryOp(Op::Id, *unknown_row)
                        }
                        (Op::Mul, Literal::Int(0)) => {
                            *self = Value::Literal(Literal::Int(0));
                        }
                        (Op::And, Literal::Bool(false)) => {
                            *self = Value::Literal(Literal::Bool(false));
                        }
                        (Op::Or, Literal::Bool(true)) => {
                            *self = Value::Literal(Literal::Bool(true));
                        }
                        _ => {}
                    }
                }
                // another weird case
                else if a1 == a2 {
                    if let Op::Sub = op {
                        *self = Value::Literal(Literal::Int(0));
                    }

                }
            }
            _ => {}
        }
    }
}


impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Literal(v1), Value::Literal(v2)) => v1 == v2,
            (Value::UnaryOp(op1, num1),
             Value::UnaryOp(op2, num2)) => (op1 == op2) && (num1 == num2),
            (Value::BinaryOp(op1, num11, num12),
             Value::BinaryOp(op2, num21, num22)) => {
                 (op1 == op2) && (num11 == num21) && (num12 == num22)
             }

              // Currently not safe to assume fns are pure without more info
              // same is true of unknown values
            _ => false
        }
    }
}


#[derive(PartialEq)]
struct Row {
    entry_num: LNum,
    value: Value,
    canonical_name: Var
}

struct Table {
    next_num: usize,
    rows: Vec<Row>,
    env: HashMap<Var, LNum>
}

impl Table {
    fn new() -> Table {
        Table {
            next_num: 0,
            rows: Vec::new(),
            env: HashMap::new()
        }
    }

    fn env(&self, var: &Var) -> LNum {
        // eprintln!("Trying to access variable {:?}", var);
        self.env[var]
    }

    fn env_nofail(&mut self, var: &Var) -> LNum {
        if !self.env.contains_key(var) {
            self.insert_value(Value::Unknown, *var)
        }
        self.env[var]
    }

    fn set_env(&mut self, var: &Var, num: LNum){
        // eprintln!("Set var {:?} equal to row {}", var, num);
        self.env.insert(*var, num);
    }

    fn get_var(&self, num: LNum) -> Var {
        self.rows[num].canonical_name
    }

    fn rewrite_var(&self, var: &Var) -> Var {
        let line = self.env(var);
        if let Value::UnaryOp(Op::Id, num) = self.get_row_value(line){
            self.get_var(*num)
        } else {
            self.get_var(line)
        }
    }

    fn insert_value(&mut self, val: Value, name: Var) {
        for row in self.rows.iter_mut() {
            if row.canonical_name == name {
                if let Value::Unknown = row.value {
                    row.value = val;
                    return
                } else {
                    panic!("Defined name twice! {}", name);
                }
            }
        }


        let new_row = Row {
            entry_num: self.next_num,
            value: val,
            canonical_name: name,
        };
        self.set_env(&name, self.next_num);
        self.next_num += 1;
        self.rows.push(new_row);
    }

    fn lookup_value(&self, val: &Value) -> Option<(LNum,Var)> {
        for row in self.rows.iter() {
            if *val == row.value {
                return Some((row.entry_num, row.canonical_name))
            }
        }
        None
    }

    fn get_row_value<'a>(&'a self, idx: LNum) -> &'a Value {
        &self.rows[idx].value
    }

    fn generate_value(&mut self, instr: &mut Instr) -> Option<Value>{
        match instr {
            Instr::Const { value, ..} =>
                Some(Value::Literal(value.clone())),
            Instr::Value { op, args, .. }
                 | Instr::Effect { op, args, .. }
                 => {
                if *op == Op::Call {
                    let args = args.iter()
                        .map(|x| {self.env_nofail(x)}).collect();
                    Some(Value::Call(args))
                } else {
                    match args.len() {
                        0 => None,
                        1 => {
                            let mut v = Value::UnaryOp(*op, self.env_nofail(&args[0]));
                            v.transform_special_form(self);
                            Some(v)
                        }
                        2 => {
                            let mut v = Value::BinaryOp(*op,
                                                self.env_nofail(&args[0]),
                                                self.env_nofail(&args[1]));
                            v.canonicalize();
                            v.transform_special_form(self);
                            Some(v)
                        },
                        _ => panic!("Operation with {} args! {:?}", args.len(), args)
                    }
                }
            }
            _  => None

        }
    }

    fn rewrite(&mut self, instr: &mut Instr) {
        let v = self.generate_value(instr);
        // eprintln!("{:?}", v);

        match instr {
            Instr::Const { dest, ..} => {
                if let Some(v) = v {
                    let linenum = self.lookup_value(&v);
                    if let Some((num, var)) = linenum {
                        // eprintln!("Value for {:?} is already present as {:?}", dest, var);
                        self.set_env(dest, num);
                    } else {
                        // eprintln!("Value for {:?} is not present", dest);
                        self.insert_value(v, *dest);
                    }
                }
            }
            Instr::Value {op, args, dest, r_type, ..} => {
                if let Some(ref v) = v {

                    let linenum = self.lookup_value(&v);

                    if let Some((num, var)) = linenum {
                        // eprintln!("Value for {:?} is already present as {:?}", dest, var);
                        self.set_env(dest, num);
                    } else {
                        // eprintln!("Value for {:?} is not present", dest);
                        self.insert_value(v.clone(), *dest);
                }
            }

                match v {
                    Some(Value::Literal(lit)) => {
                        *instr = Instr::Const { op: Op::Const,
                                               dest: *dest,
                                               r_type: r_type.clone(),
                                               value: lit}
                    }
                    Some(Value::UnaryOp(new_op, new_arg)) => {
                        *op = new_op;
                        *args = vec! [self.get_var(new_arg)]
                    }
                    _ => {
                        for arg in args.iter_mut() {
                            *arg = self.rewrite_var(arg)
                        }
                    }
                }

            }
            Instr::Effect {args, ..} => {
                for arg in args.iter_mut() {
                    *arg = self.rewrite_var(arg)
                }
            }
            _ => {}
        }

    }
}

pub fn run_lvn(instrs: &mut Vec<Instr>) {
    // eprintln!("{:?}", instrs);
    force_unique_names(instrs);
    // for instr in instrs.iter(){
    //     eprintln!("{}", instr);
    // }
    // eprintln!("\n\n\n");

    let mut tbl = Table::new();

    for instr in instrs.iter_mut() {
        tbl.rewrite(instr)
    }

}


fn force_unique_names(instrs: &mut Vec<Instr>) {
    let mut name_reader = namer();
    let mut new_mapping = HashMap::<Var, Vec<(RangeInclusive<usize>, Var)>>::new();
    let mut prev_defn = HashMap::<Var, usize>::new();


    for (idx, instr) in instrs.iter().enumerate() {
        match instr {
             Instr::Const { dest, .. } | Instr::Value { dest, ..} => {
                if prev_defn.contains_key(dest) && new_mapping.contains_key(dest){
                    let prior_marker = prev_defn.remove(dest).unwrap();
                    let fresh = name_reader.fresh(&dest.0);

                    new_mapping.get_mut(dest).unwrap().push((prior_marker..=idx, Var(fresh)));
                    prev_defn.insert(*dest, idx);
                } else if prev_defn.contains_key(dest) {
                    let prior_marker = prev_defn.remove(dest).unwrap();
                    let fresh = name_reader.fresh(&dest.0);

                    let new_list = vec! [(prior_marker..=idx, Var(fresh))];
                    new_mapping.insert(*dest, new_list);
                    prev_defn.insert(*dest, idx);
                } else {
                    prev_defn.insert(*dest, idx);
                }
             }
             _ => {}
        }
    }


    for (idx, instr) in instrs.iter_mut().enumerate() {
        match instr {
            Instr::Const { dest, .. } => {
                if new_mapping.contains_key(dest) {
                    let remappings: Vec<&(RangeInclusive<usize>, Var)> = new_mapping.get(dest).unwrap()
                        .iter().filter(|&(x, ..)| {*x.start() == idx}).collect();
                    if remappings.len() == 1 {
                    let (range, new_name) = remappings[0];
                        *dest = *new_name;
                    }
                }
            }
            Instr::Effect { args, ..} => {
                for var in args.iter_mut() {
                    if new_mapping.contains_key(var) {
                        let remappings: Vec<&(RangeInclusive<usize>, Var)> = new_mapping.get(var).unwrap()
                         .iter().filter(|&(x, ..)| {x.contains(&idx)}).collect();

                         // Since this is not defining a variable, it
                         // can't be in more than one range
                         if !remappings.is_empty() {
                            let (_, new_name) = remappings[0];

                            *var = *new_name;
                         }

                         // if there is no mapping then the variable is already
                         // correctly named

                    }
                }
            }
            Instr::Value { dest, args, .. } => {
                if new_mapping.contains_key(dest) {
                    let remappings: Vec<&(RangeInclusive<usize>, Var)> = new_mapping.get(dest).unwrap()
                        .iter().filter(|&(x, ..)| {*x.start() == idx}).collect();

                    // eprintln!("On line {}, there are {} mappings", idx, remappings.len());
                    if remappings.len() == 1 {
                    let (range, new_name) = remappings[0];
                    *dest = *new_name;
                    }
                }

                for var in args.iter_mut() {
                    if new_mapping.contains_key(var) {
                        let remappings: Vec<&(RangeInclusive<usize>, Var)> = new_mapping.get(var).unwrap()
                         .iter().filter(|&(x, ..)| {x.contains(&idx) && *x.start() != idx}).collect();


                        // There should be exactly one range satisfying
                        // the above requirements, I hope.

                        if !remappings.is_empty() {

                        let (_, new_name) = remappings[0];

                        *var = *new_name;
                        }
                    }
                }
            }
            _ => {}
        }

    }
}
