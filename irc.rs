use std::io::*;
use connection::*;
use writer::*;
use std::collections::hashmap::HashMap;

// Single server for now.
pub struct IrcConfig<'a> {
    pub host: &'a str,
    pub port: u16,
    pub channels: Vec<&'a str>,
    pub nick: &'a str,
    pub descr: &'a str,
}

// A regular irc message sent from the server.
struct IrcMsg {
    prefix: String,
    code: String,
    param: String,
}

impl IrcMsg {
    fn new(s: &str) -> Option<IrcMsg> {
        println!("matching: {}", s);
        let re = regex!(r"^(:\w+)?\s*(\w+)\s+(.*)\r?$");
        let caps = re.captures(s);
        match caps {
            Some(x) => {
                Some(IrcMsg {
                    prefix: x.at(1).to_string(),
                    code: x.at(2).to_string(),
                    param: x.at(3).to_string(),
                })
            },
            None => None
        }
    }
}

// command callbacks?
// + register functionsto send to irc.
pub struct Irc<'a> {
    // Connections to irc server and over internal channel.
    conn: ServerConnection,
    tx: Sender<ConnectionEvent>,
    rx: Receiver<ConnectionEvent>,

    // Bot info.
    nick: String,
    descr: String,

    //raw_cb: HashMap<String, fn(s: String) -> Option<String>>,
    raw_cb: Vec<|s: String|:'a -> Option<String>>,
    //raw_cb: Vec<||:'a -> ()>,
}

// FIXME move?
fn ping(s: String) -> Option<String> {
    let re = regex!(r"^PING\s(.+)$");
    let caps = re.captures(s.as_slice());
    match caps {
        Some(x) => Some(format!("PONG {}", x.at(1))),
        None => None,
    }
}

impl<'a> Irc<'a> {
    // Create a new irc instance and connect to the server, but don't act on it.
    pub fn connect(conf: IrcConfig) -> Irc {
        let (tx, rx) = channel();
        let mut irc = Irc {
            conn: ServerConnection::new(conf.host, conf.port),
            tx: tx,
            rx: rx,
            nick: conf.nick.to_string(),
            descr: conf.descr.to_string(),
            raw_cb: Vec::new(),
        };

        irc.raw_cb.push(|s: String| -> Option<String> {
            println!("raw cb: {}", s);
            None
        });

        irc
    }

    // Construct a writer we can use to send things to irc.
    // Uses a channel transmitter with a process in the backround.
    pub fn writer(&self) -> IrcWriter {
        IrcWriter::new(self.tx.clone())
    }

    // Called when we receive a response from the server.
    fn handle_received(&self, line: &String) {
        // FIXME don't know how to call callbacks
        // Need &mut self, but that interfereces with rx looping.
        //for cb in self.raw_cb.mut_iter() {
            //(*cb)();
        //}

        // Trim away newlines and unneeded spaces.
        let s = line.as_slice().trim().to_string();
        println!("< {}", s);

        // FIXME filter output (not callbacks)
        // blacklist these
        // 001, 002, 003, 004       greetings etc
        // 005                      supported things
        // 251, 252, 253, 254, 255  server status, num connections etc
        // 372, 375, 376            MOTD
        // NOTICE                   crap?

        let writer = self.writer();

        // FIXME do this for all raw callbacks
        match ping(s) {
            Some(x) => writer.write_line(x),
            _ => (),
        }
    }

    // Run irc client and block until done.
    pub fn run(&mut self) {
        self.spawn_reader();
        self.run_handler();
    }

    // Spawn a proc reader which listens to incoming messages from irc.
    fn spawn_reader(&self) {
        println!("Spawning irc reader");
        let tcp = self.conn.tcp.clone(); // Workaround to avoid irc capture
        let tx = self.tx.clone();
        spawn(proc() {
            let mut reader = BufferedReader::new(tcp);
            loop {
                match read_line(&mut reader) {
                    Some(x) => tx.send(Received(x)),
                    None => break,
                }
            }
            println!("Quitting irc reader");
        });
    }

    // FIXME spawn a thread instead?
    fn run_handler(&mut self) {
        println!("Running event handler");
        let tcp = self.conn.tcp.clone();
        let mut stream = LineBufferedWriter::new(tcp.clone());

        // Start with identifying
        let writer = self.writer();
        writer.identify(&self.nick, &self.descr);

        // Loop and handle in and output events.
        // Quit is a special case to allow us to close the program.
        for x in self.rx.iter() {
            match x {
                Output(ref s) => {
                    // FIXME method for this?
                    println!("> {}", s);
                    write_line(&mut stream, s.as_slice());
                },
                Received(ref s) => {
                    self.handle_received(s);
                },
                Quit => {
                    self.conn.close();
                    break;
                },
            }
        }
        println!("Exiting irc writer");
    }
}

//struct IrcMsg {
    //prefix: String,
    //code: String,
    //param: String,
//}

//struct IrcPrivMsg {
    //channel: String,
    //msg: String,
//}

//struct IrcCmdMsg {
    //channel: String,
    //cmd: String,
    //args: String,
//}

//struct Cmd {
    //name: String,
    //args: String,
//}

mod tests {
    // Test irc message matching
    #[test]
    fn msg() {
        some_msg(":pref 020 rustbot lblblb", ":pref", "020", "rustbot lblblb");
        some_msg("020 rustbot lblblb", "", "020", "rustbot lblblb");
        none_msg("a");
    }

    // Test callbacks
    #[test]
    fn ping() {
        test_cb_match(super::ping, "PING :423131321", "PONG :423131321");
        test_cb_none(super::ping, "JOIN :asdf");
    }

    // IRC message parsing test functions
    #[cfg(test)]
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

    #[cfg(test)]
    fn none_msg(s: &str) {
        match super::IrcMsg::new(s) {
            Some(_) => fail!("Matched {}, s"),
            None => (),
        }
    }

    // Raw callback test functions
    #[cfg(test)]
    fn test_cb_match(f: |String| -> Option<String>, s: &str, expected: &str) {
        match f(s.to_string()) {
            Some(got) => assert_eq!(got, expected.to_string()),
            None => fail!("None"),
        }
    }

    #[cfg(test)]
    fn test_cb_none(f: |String| -> Option<String>, s: &str) {
        match f(s.to_string()) {
            Some(_) => fail!("Some"),
            None => (),
        }
    }
}

