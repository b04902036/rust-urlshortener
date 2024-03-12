use std::collections::HashMap;

lazy_static::lazy_static! {
    static ref BASE62_MAPPER: HashMap<i32, char> = {
        let mut map = HashMap::new();
        let base62_chars = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyzzz".chars();
        for (index, character) in base62_chars.enumerate() {
            map.insert(index as i32, character);
        }
        map
    };
}

pub fn map(idx: i32) -> char {
    BASE62_MAPPER[&(idx & 63)]
}
