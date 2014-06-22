use regex::Regex;

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

