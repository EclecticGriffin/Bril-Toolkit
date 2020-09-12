use bimap::BiHashMap;
use std::cell::RefCell;
use std::sync::{Arc, Mutex, Once};
use std::mem::transmute;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Name(u64);

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
    pub fn remove_and_return_string(&self, name: &Name) -> String {
        (*self.mapper.as_ref().lock().unwrap()).remove_and_return_string(name)
    }

    // pub fn lock(&self) -> {
    //     self.mapper.lock().unwrap()
    // }
}

struct NameMapper {
    next_name: Name,
    map: BiHashMap<String, Name>
}

impl NameMapper {
    fn new() -> NameMapper {
        NameMapper {
            next_name:  Name(0),
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

    fn remove_and_return_string(&mut self, name: &Name) -> String {
        let (s, n) = self.map.remove_by_right(name).unwrap();
        s
    }
}


pub fn namer() -> NameReader {
    static mut NAMEREADER: *const NameReader = 0 as *const NameReader;
    static ONCE: Once = Once::new();

    unsafe {
        ONCE.call_once(|| {
            let reader = NameReader::new();
            NAMEREADER = transmute(Box::new(reader));
        });
    (*NAMEREADER).clone()
    }
}
