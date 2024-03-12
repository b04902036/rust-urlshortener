use super::base62;
use crate::config;
use rand::{thread_rng, RngCore};

const BUF_LENGTH: usize = (config::SHORT_URL_LENGTH * 6 + (8 - 1)) / 8;

pub fn gen_short_url() -> String {
    let mut buf = [0u8; BUF_LENGTH];
    thread_rng().fill_bytes(&mut buf);

    let mut length_counter = 0;
    let mut now = 0;
    let mut idx = 0;
    let mut ret = String::new();
    for _ in 0..config::SHORT_URL_LENGTH {
        if length_counter < 6 {
            now <<= 8;
            now += buf[idx] as i32;
            idx += 1;
            length_counter += 8;
        }
        ret.push(base62::map(now));
        now >>= 6;
        length_counter -= 6;
    }
    ret
}
