
// Single server for now.
pub struct IrcConfig<'a> {
    pub host: &'a str,
    pub port: u16,
    pub nick: &'a str,
    pub descr: &'a str,
    pub blacklist: Vec<&'a str>,
}
