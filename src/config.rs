use std::collections::HashMap;
use clap::Values;
pub const allowed_values: &[&str] = &["all", "g_tdce", "l_tdce", "lvn", "orph", "solo_lvn"];

pub enum LVNChoice {
    Solo,
    Bool(bool)
}

impl LVNChoice {
    pub fn run_lvn(&self) -> bool {
        match self {
            LVNChoice::Solo => true,
            LVNChoice::Bool(b) => *b
        }
    }

    pub fn run_solo(&self) -> bool {
        match self {
            LVNChoice::Solo => true,
            LVNChoice::Bool(b) => false
        }
    }

    pub fn run_normal(&self) -> bool {
        match self {
            LVNChoice::Solo => false,
            LVNChoice::Bool(b) => *b
        }
    }
}

pub struct ConfigOptions {
    pub orphan_block: bool,
    pub l_tdce: bool,
    pub g_tdce: bool,
    pub lvn: LVNChoice
}

impl ConfigOptions {
    fn config_map(options: Values) -> HashMap<&str, bool> {
        let mut hash = HashMap::<&str, bool>::new();
        for opt in options {
            hash.insert(opt, true);
        }

        if hash.contains_key("all") {
            for key in allowed_values {
                hash.insert(&key, true);
            }
        } else {
            for key in allowed_values {
                if !hash.contains_key(key) {
                    hash.insert(&key, false);
                }
            }
        }

        hash
    }

    pub fn new(options: Values) -> ConfigOptions {
        let map = ConfigOptions::config_map(options);

        let mut lvn = LVNChoice::Bool(map["lvn"]);
        if map["solo_lvn"] && !map["lvn"] {
            lvn = LVNChoice::Solo
        }
        ConfigOptions {
            orphan_block: map["orph"],
            l_tdce: map["l_tdce"],
            g_tdce: map["g_tdce"],
            lvn
        }
    }
}
