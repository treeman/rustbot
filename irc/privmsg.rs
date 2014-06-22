use core::fmt::{Show, Formatter, Result};

use irc::msg::*;

// FIXME how to handle actual private messages?
// A privmsg sent from the server.
pub struct IrcPrivMsg {
    pub orig: String,
    pub sender_nick: String,
    pub sender_info: String,
    pub channel: String,
    pub msg: String,
}

impl IrcPrivMsg {
    pub fn new(irc_msg: &IrcMsg) -> Option<IrcPrivMsg> {
        if irc_msg.code.as_slice() == "PRIVMSG" {
            match (match_sender(irc_msg), match_message(irc_msg)) {
                (Some((nick, info)), Some((channel, msg))) =>
                    Some(IrcPrivMsg {
                        orig: irc_msg.orig.clone(),
                        sender_nick: nick,
                        sender_info: info,
                        channel: channel,
                        msg: msg,
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
               self.sender_nick, self.sender_info, self.channel, self.msg)
    }
}

// Fetch nick + sender info
fn match_sender(msg: &IrcMsg) -> Option<(String, String)> {
    let re = regex!(r":([^!]+)(?:!(.+))?");
    let caps = re.captures(msg.prefix.as_slice());
    match caps {
        Some(x) => Some((x.at(1).to_string(), x.at(2).to_string())),
        None => None,
    }
}

// Fetch channel + message
fn match_message(msg: &IrcMsg) -> Option<(String, String)> {
    let re = regex!(r"(#\S+)\s+:(.*)");
    let caps = re.captures(msg.param.as_slice());
    match caps {
        Some(x) => Some((x.at(1).to_string(), x.at(2).to_string())),
        None => None,
    }
}


#[cfg(test)]
mod tests {
    use IrcMsg = irc::msg::IrcMsg;
    use IrcPrivMsg = super::IrcPrivMsg;

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

    fn some_msg(s: &str, sender_nick: &str, sender_info: &str, channel: &str, msg: &str) {
        // Very ugly! :(
        match IrcMsg::new(s) {
            Some(irc_msg) => {
                match IrcPrivMsg::new(&irc_msg) {
                    Some(x) => {
                        assert_eq!(x.sender_nick, sender_nick.to_string());
                        assert_eq!(x.sender_info, sender_info.to_string());
                        assert_eq!(x.channel, channel.to_string());
                        assert_eq!(x.msg, msg.to_string());
                    },
                    None => fail!("Did not match {}", s),
                }
            }
            None => fail!("Did not match {}", s),
        }
    }

    fn none_msg(s: &str) {
        // Is there a prettier way?
        match IrcMsg::new(s) {
            Some(msg) => {
                match IrcPrivMsg::new(&msg) {
                    Some(_) => fail!("Matched {}, s"),
                    None => (),
                }
            }
            None => (),
        }
    }
}

