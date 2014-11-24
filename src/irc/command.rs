//use core::fmt::{Show, Formatter, Result};

use irc::privmsg::*;
use util::*;

// Command through irc.
#[deriving(Show)]
pub struct IrcCommand<'a> {
    pub name: &'a str,
    pub args: Vec<&'a str>,
    //pub msg: &'a IrcPrivMsg,
    pub channel: &'a str,
    // TODO arg string, everything else a string
}

// TODO something like this?
impl<'a> IrcCommand<'a> {
    pub fn new(msg: &'a IrcPrivMsg, key: char) -> Option<IrcCommand<'a>> {
        match Command::new(msg.txt[], key) {
            Some(cmd) => {
                Some(IrcCommand {
                    name: cmd.name,
                    args: cmd.args,
                    //msg: msg,
                    channel: msg.channel[],
                })
            },
            None => None,
        }
    }
}

// An actual command.
// Structured like.
// .cmd arg1 arg2
// Can be from irc or whatever.
#[deriving(Show)]
pub struct Command<'a> {
    pub name: &'a str,
    pub args: Vec<&'a str>,
}

impl<'a> Command<'a> {
    //<char>name arg1 arg2
    //ex:
    //.print 1 2
    pub fn new(s: &'a str, key: char) -> Option<Command<'a>> {
        let s = s.trim();
        if s.len() > 0 && s.char_at(0) == key {
            let split = space_split(s);
            let name = split[0].slice_from(1);
            let mut args = Vec::new();
            args.push_all(split.slice_from(1));

            Some(Command {
                name: name,
                args: args,
            })
        }
        else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn command() {
        cmd(".a", "a", vec![]);
        cmd(".a b c", "a", vec!["b", "c"]);
        cmd(".foo bar", "foo", vec!["bar"]);
        cmd("   .foo   bar  ", "foo", vec!["bar"]);
    }

    fn cmd<'a>(s: &'a str, name: &str, args: Vec<&'a str>) {
        let key = '.';
        let cmd = super::Command::new(s, key);
        match cmd {
            Some(x) => {
                assert_eq!(x.name, name);
                assert_eq!(x.args, args);
            },
            None => panic!("No match"),
        }
    }
}

