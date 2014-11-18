use core::fmt::{Show, Formatter, Result};

use irc::msg::*;

// FIXME how to handle actual private messages?
// A privmsg sent from the server.
pub struct IrcPrivMsg {
    pub orig: String,
    pub sender_nick: String,
    pub sender_info: String,
    pub channel: String,
    pub txt: String,
}

impl IrcPrivMsg {
    pub fn new(msg: &IrcMsg) -> Option<IrcPrivMsg> {
        if msg.code[] == "PRIVMSG" {
            match (msg.match_sender(), msg.match_message()) {
                (Some((nick, info)), Some((channel, txt))) =>
                    Some(IrcPrivMsg {
                        orig: msg.orig.clone(),
                        sender_nick: nick,
                        sender_info: info,
                        channel: channel,
                        txt: txt,
                    }),
                _ => None,
            }
        }
        else {
            None
        }
    }
}

impl Show for IrcPrivMsg {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "sender: {} ({}) channel: {} msg: {}",
               self.sender_nick, self.sender_info, self.channel, self.txt)
    }
}


#[cfg(test)]
mod tests {
    //use IrcMsg = irc::msg::IrcMsg;
    //use IrcPrivMsg = super::IrcPrivMsg;
    use irc::msg::IrcMsg;
    use super::IrcPrivMsg;

    // Test irc message matching
    #[test]
    fn msg() {
        some_msg(":Mowah!~Mowah@who.se PRIVMSG #treecraft :yo rustie! :) #.#",
                 "Mowah", "~Mowah@who.se", "#treecraft", "yo rustie! :) #.#");
        some_msg(":Mowah PRIVMSG #treecraft :yo rustie! :) #.#",
                 "Mowah", "", "#treecraft", "yo rustie! :) #.#");
        none_msg(":pref 020 rustbot lblblb");
        none_msg("020 rustbot lblblb");
        none_msg(":dreamhack.se.quakenet.org 376 rustbot :End of /MOTD command");
        none_msg("a");
        none_msg(":underworld2.no.quakenet.org 221 rustbot +i");
    }

    fn some_msg(s: &str, sender_nick: &str, sender_info: &str, channel: &str, txt: &str) {
        // Very ugly! :(
        match IrcMsg::new(s) {
            Some(irc_msg) => {
                match IrcPrivMsg::new(&irc_msg) {
                    Some(x) => {
                        assert_eq!(x.sender_nick, sender_nick.to_string());
                        assert_eq!(x.sender_info, sender_info.to_string());
                        assert_eq!(x.channel, channel.to_string());
                        assert_eq!(x.txt, txt.to_string());
                    },
                    None => panic!("Did not match {}", s),
                }
            }
            None => panic!("Did not match {}", s),
        }
    }

    fn none_msg(s: &str) {
        // Is there a prettier way?
        match IrcMsg::new(s) {
            Some(msg) => {
                match IrcPrivMsg::new(&msg) {
                    Some(_) => panic!("Matched {}, s"),
                    None => (),
                }
            }
            None => (),
        }
    }
}

