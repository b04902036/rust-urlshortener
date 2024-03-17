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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base62_mapper_with_64_length() {
        assert_eq!(BASE62_MAPPER.len(), 64);
    }

    #[test]
    fn map_should_not_overflow() {
        for i in 0..1000 {
            _ = map(i);
        }
    }
}
