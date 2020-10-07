use bimap::BiHashMap;
use std::sync::{Arc, Mutex, Once};
use std::mem::transmute;

use serde::de::{self, Deserializer, Deserialize, Visitor};
use serde::{Serialize, Serializer};
use std::fmt::{self, Display};
use super::wrapper_names::Label;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Ord, PartialOrd)]
pub struct Name(pub u64);

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", namer().get_string(&self))
    }
}

impl<'de> Deserialize<'de> for Name {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de> {

            struct NameVisitor;

            impl<'de> Visitor<'de> for NameVisitor {
                type Value = Name;

                fn visit_str<E>(self, value: &str) -> Result<Name,E>
                where
                    E: de::Error
                {
                    let namer = namer();
                    Ok(namer.get_name(String::from(value)))
                }

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("`name`")
                }
    }
    deserializer.deserialize_identifier(NameVisitor)
    }
}

impl Serialize for Name {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let namer = namer();
        let string = namer.get_string(&self);
        serializer.serialize_str(&string)
    }
}

#[derive(Clone)]
pub struct NameReader {
    mapper: Arc<Mutex<NameMapper>>
}
impl<'a> NameReader {
    fn new() -> Self {
        Self {
            mapper: Arc::new(Mutex::new(NameMapper::new()))
        }
    }

    pub fn get_name(&self, key: String) -> Name {
        (*self.mapper.as_ref().lock().unwrap()).get_name(key)
    }
    pub fn get_string(&self, name: &Name) -> String {
        let map = &*self.mapper.as_ref().lock().unwrap();
        map.get_string(name).clone()
    }

    pub fn fresh(&self, base: &Name) -> Name {
        let map = &mut *self.mapper.as_ref().lock().unwrap();
        let str_form = map.get_string(base).clone();
        map.gen_fresh_name(&str_form)
    }

    pub fn fresh_label(&self) -> Label {
        let map = &mut *self.mapper.as_ref().lock().unwrap();
        Label(map.gen_fresh_name(&String::from("tmp_label")))
    }
    // pub fn remove_and_return_string(&self, name: &Name) -> String {
    //     (*self.mapper.as_ref().lock().unwrap()).remove_and_return_string(name)
    // }

}

struct NameMapper {
    next_name: Name,
    map: BiHashMap<String, Name>
}

impl NameMapper {
    fn new() -> NameMapper {
        NameMapper {
            next_name: Name(0),
            map: BiHashMap::<String, Name>::new()
        }
    }

    fn get_name(&mut self, key: String) -> Name {
        if self.map.contains_left(&key) {
            *self.map.get_by_left(&key).unwrap()
        } else {
            self.map.insert(key, self.next_name);
            let old = self.next_name;
            self.next_name = Name(self.next_name.0 + 1);
            old
        }
    }

    fn get_string(&self, name: &Name) ->  &String {
        self.map.get_by_right(name).unwrap()
    }

    fn gen_fresh_name(&mut self, base: &str) -> Name {
        let mut counter = 1;
        let mut fresh = format!("{}_{}", base, counter);
        while self.map.contains_left(&fresh) {
            counter += 1;
            fresh = format!("{}_{}", base, counter);
        };

        self.get_name(fresh)
    }
    // fn remove_and_return_string(&mut self, name: &Name) -> String {
    //     let (s, _n) = self.map.remove_by_right(name).unwrap();
    //     s
    // }
}

// based entirely on how stdin is handled
pub fn namer() -> NameReader {
    static mut NAMEREADER: *const NameReader = 0 as *const NameReader;
    static ONCE: Once = Once::new();

    unsafe {
        ONCE.call_once(|| {
            let reader = NameReader::new();
            NAMEREADER = transmute(Box::new(reader)); // !!!!
        });
    (*NAMEREADER).clone()
    }
}
