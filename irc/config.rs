use regex::Regex;
use serialize::{json, Decodable};
use std::io::{File, Open, Read};

// Single server for now.
pub struct IrcConfig<'a> {
    pub host: &'a str,
    pub port: u16,
    pub nick: &'a str,
    pub descr: &'a str,
    pub channels: Vec<&'a str>,
    pub in_blacklist: Vec<&'a str>,
    pub out_blacklist: Vec<Regex>,
    pub cmd_prefix: char,
}

#[deriving(Decodable)]
pub struct JsonConfig {
    pub host: String,
    pub port: u16,
    pub nick: String,
    pub descr: String,
    pub channels: Vec<String>,
    pub in_blacklist: Vec<String>,
    pub out_blacklist: Vec<String>,
    pub cmd_prefix: char
}

impl JsonConfig {
    pub fn new(location: String) -> JsonConfig {
        let p = Path::new(location.as_slice());
        let mut file = match File::open_mode(&p, Open, Read) {
            Ok(f) => f,
            Err(e) => fail!("file error: {}", e)
        };

        let decoded: String = match file.read_to_str() {
            Ok(f) => f,
            Err(e) => fail!("file error: {}", e)
        };

        let json_object = json::from_str(decoded.as_slice());
        let mut decoder = json::Decoder::new(json_object.unwrap());

        return match Decodable::decode(&mut decoder) {
            Ok(v) => v,
            Err(e) => fail!("Decoding error: {}", e)
        };
    }
}
