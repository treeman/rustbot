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

    // Fetch nick + sender info
    pub fn match_sender(&self) -> Option<(String, String)> {
        let re = regex!(r":([^!]+)(?:!(.+))?");
        let caps = re.captures(self.prefix[]);
        match caps {
            Some(x) => Some((x.at(1).to_string(), x.at(2).to_string())),
            None => None,
        }
    }

    // Fetch channel + message
    pub fn match_message(&self) -> Option<(String, String)> {
        let re = regex!(r"(#\S+)\s+:(.*)");
        let caps = re.captures(self.param[]);
        match caps {
            Some(x) => Some((x.at(1).to_string(), x.at(2).to_string())),
            None => None,
        }
    }
}

impl Show for IrcMsg {
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
            None => panic!("Did not match {}", s),
        }
    }

    fn none_msg(s: &str) {
        match super::IrcMsg::new(s) {
            Some(_) => panic!("Matched {}, s"),
            None => (),
        }
    }
}

