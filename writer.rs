use connection::*;

// Convenience wrapper to abstract away write commands.
pub struct IrcWriter {
    tx: Sender<ConnectionEvent>,
}

impl IrcWriter {
    // Wrapping a tx channel.
    pub fn new(tx: Sender<ConnectionEvent>) -> IrcWriter {
        IrcWriter{ tx: tx.clone() }
    }

    // Join a channel.
    pub fn join(&self, chan: &String) {
        self.write_line(format!("JOIN {}", chan));
    }

    // Identify us to the server.
    pub fn identify(&self, nick: &String, descr: &String) {
        self.write_line(format!("NICK {}", nick));
        self.write_line(format!("USER {} 8 * :{}", nick, descr));
    }

    // Change nickname.
    pub fn nick(&self, s: String) {
        self.write_line(format!("NICK {}", s));
    }

    // Use for general output.
    pub fn write_line(&self, s: String) {
        self.tx.send(Output(s));
    }

    // Use for closing down.
    pub fn send_quit(&self, s: String) {
        self.tx.send(Output(format!("QUIT :{}", s)));
        self.tx.send(Quit);
    }
}

