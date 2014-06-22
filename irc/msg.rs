use core::fmt::{Show, Formatter, Result};

// A regular irc message sent from the server.
pub struct IrcMsg<'a> {
    pub orig: String,
    pub prefix: String,
    pub code: String,
    pub param: String,
}

impl<'a> IrcMsg<'a> {
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

impl<'a> Show for IrcMsg<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "prefix: {} code: {} param: {}",
               self.prefix, self.code, self.param)
    }
}

#[cfg(test)]
mod tests {
    // Test irc message matching
    #[test]
    fn msg() {
        some_msg(":pref 020 rustbot lblblb", ":pref", "020", "rustbot lblblb");
        some_msg("020 rustbot lblblb", "", "020", "rustbot lblblb");
        some_msg(":dreamhack.se.quakenet.org 376 rustbot :End of /MOTD command",
                 ":dreamhack.se.quakenet.org", "376", "rustbot :End of /MOTD command");
        none_msg("a");
    }

    fn some_msg(s: &str, prefix: &str, code: &str, param: &str) {
        match super::IrcMsg::new(s) {
            Some(x) => {
                assert_eq!(x.prefix, prefix.to_string());
                assert_eq!(x.code, code.to_string());
                assert_eq!(x.param, param.to_string());
            },
            None => fail!("Did not match {}", s),
        }
    }

    fn none_msg(s: &str) {
        match super::IrcMsg::new(s) {
            Some(_) => fail!("Matched {}, s"),
            None => (),
        }
    }
}

