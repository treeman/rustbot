use irc::connection::*;
use util::*;

// Convenience wrapper to abstract away write commands.
pub struct IrcWriter {
    tx: Sender<ConnectionEvent>,
}

// FIXME take T: ToStr as input instead
impl IrcWriter {
    // Wrapping a tx channel.
    pub fn new(tx: Sender<ConnectionEvent>) -> IrcWriter {
        IrcWriter { tx: tx.clone() }
    }

    // Join a channel.
    pub fn join(&self, chan: &str) {
        self.output(format!("JOIN {}", chan));
    }

    // Identify us to the server.
    pub fn identify(&self, nick: &str, descr: &str) {
        self.output(format!("NICK {}", nick));
        self.output(format!("USER {} 8 * :{}", nick, descr));
    }

    // Change nickname.
    pub fn nick(&self, s: &str) {
        self.output(format!("NICK {}", s));
    }

    // Send a PRIVMSG.
    pub fn msg(&self, target: &str, msg: &str) {
        for line in newline_split(msg).iter() {
            self.output(format!("PRIVMSG {} :{}", target, line));
        }
    }

    // Use for general output.
    pub fn output(&self, s: String) {
        // FIXME pastebin http://pastebin.com/api
        // when we have too many lines.
        // FIXME throttle
        let lines = newline_split(s.as_slice());
        for line in lines.iter() {
            self.tx.send(Output(line.to_string()));
        }
    }

    // Use for closing down.
    pub fn quit(&self, s: &str) {
        self.tx.send(Output(format!("QUIT :{}", s)));
        self.tx.send(Quit);
    }
}

