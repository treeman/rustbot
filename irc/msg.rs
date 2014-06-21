use core::fmt::{Show, Formatter, Result};

// A regular irc message sent from the server.
pub struct IrcMsg {
    pub orig: String,
    pub prefix: String,
    pub code: String,
    pub param: String,
}

impl IrcMsg {
    pub fn new(s: &str) -> Option<IrcMsg> {
        let re = regex!(r"^(:\S+)?\s*(\S+)\s+(.*)\r?$");
        let caps = re.captures(s);
        match caps {
            Some(x) => {
                Some(IrcMsg {
                    orig: s.to_string(),
                    prefix: x.at(1).to_string(),
                    code: x.at(2).to_string(),
                    param: x.at(3).to_string(),
                })
            },
            None => None
        }
    }
}

impl Show for IrcMsg {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "prefix: {} code: {} param: {}",
               self.prefix, self.code, self.param)
    }
}
