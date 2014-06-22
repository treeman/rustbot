use std::io::*;
//use regex::*;
use std::collections::hashmap::HashSet;
use std::collections::hashmap::HashMap;

use irc::config::*;
use irc::connection::*;
use irc::msg::IrcMsg;
use irc::writer::*;
use irc::info::BotInfo;

pub mod config;
pub mod connection;
pub mod msg;
pub mod writer;
mod info;

pub struct Irc<'a> {
    // Connections to irc server and over internal channel.
    conn: ServerConnection,

    // FIXME Need to store tx/rx somehow if we want to be able to expose
    // writer (I think?)
    //tx: Sender<ConnectionEvent>,
    //rx: Receiver<ConnectionEvent>,

    // General config
    info: BotInfo<'a>,
    blacklist: HashSet<String>, // String to avoid lifetime issues :)

    // Callbacks at received events
    raw_cb: Vec<|s: &str, writer: &IrcWriter, info: &BotInfo|:'a>,

    // This is a workaround for a multimap.
    code_cb: HashMap<String, Vec<|msg: &IrcMsg, writer: &IrcWriter, info: &BotInfo|:'a>>,
}

impl<'a> Irc<'a> {
    // Create a new irc instance and connect to the server, but don't act on it.
    pub fn connect<'b>(conf: IrcConfig<'b>) -> Irc<'b> {

        let mut blacklist = HashSet::new();
        for x in conf.blacklist.iter() {
            blacklist.insert(x.to_string());
        }

        let mut irc = Irc {
            conn: ServerConnection::new(conf.host, conf.port),
            //tx: tx,
            //rx: rx,
            info: BotInfo::new(&conf),
            blacklist: blacklist,

            raw_cb: Vec::new(),
            code_cb: HashMap::new(),
        };

        irc.register_default_callbacks();
        irc
    }

    // Register a callback for a specific irc msg code.
    pub fn register_code_cb(&mut self, code: &str,
                            cb: |msg: &IrcMsg, writer: &IrcWriter, info: &BotInfo|:'a)
    {
        let c = code.to_string();
        if !self.code_cb.contains_key(&c) {
            self.code_cb.insert(c.clone(), Vec::new());
        }
        let cbs = self.code_cb.get_mut(&c);
        cbs.push(cb);
    }

    fn register_default_callbacks(&mut self) {
        self.register_code_cb("PING", |msg: &IrcMsg, writer: &IrcWriter, _| {
            writer.write_line(format!("PONG {}", msg.param));
        });

        // Key on 004, should be fine as it's usually in the beginning I believe?
        self.register_code_cb("004", |_, writer: &IrcWriter, info: &BotInfo| {
            for chan in info.channels.iter() {
                writer.join(*chan);
            }
        });
    }

    // Construct a writer we can use to send things to irc.
    // Uses a channel transmitter with a process in the backround.
    //pub fn writer(&self) -> IrcWriter {
        //IrcWriter::new(self.tx.clone())
    //}

    // Called when we have a properly formatted irc message.
    fn handle_msg(&mut self, msg: &IrcMsg, writer: &IrcWriter) {
        // Print received message if it's not blacklisted.
        let code = msg.code.clone();
        if !self.blacklist.contains(&code) {
            println!("< {}", msg.orig);
        }

        // Irc msg callbacks
        let c = msg.code.clone();
        if self.code_cb.contains_key(&c) {
            let cbs = self.code_cb.get_mut(&c);
            for cb in cbs.mut_iter() {
                (*cb)(msg, writer, &self.info);
            }
        }

        // FIXME callbacks for privmsg
        // FIXME look for commands
    }

    // Called when we receive a response from the server.
    fn handle_received(&mut self, line: &String, writer: &IrcWriter) {
        // Trim away newlines and unneeded spaces.
        let s = line.as_slice().trim();

        for cb in self.raw_cb.mut_iter() {
            (*cb)(s, writer, &self.info);
        }

        match IrcMsg::new(s) {
            Some(msg) => {
                // Print inside here so we can skip certain codes.
                self.handle_msg(&msg, writer);
            },
            _ => {
                // Couldn't capture message, print it here.
                println!("<! {}", s);
            },
        }
    }

    // Actually write something to irc.
    fn handle_write(&self, s: &String, stream: &mut LineBufferedWriter<TcpStream>) {
        println!("> {}", s);
        write_line(stream, s.as_slice());
    }

    // Run irc client and block until done.
    pub fn run(&mut self) {
        let (tx, rx) = channel();
        self.spawn_reader(tx.clone());
        self.run_handler(tx.clone(), rx);
    }

    // Spawn a proc reader which listens to incoming messages from irc.
    fn spawn_reader(&self, tx: Sender<ConnectionEvent>) {
        println!("Spawning irc reader");
        let tcp = self.conn.tcp.clone(); // Workaround to avoid irc capture
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

    // Run event handler. Will block.
    fn run_handler(&mut self, tx: Sender<ConnectionEvent>, rx: Receiver<ConnectionEvent>) {
        println!("Spawning event handler");
        let tcp = self.conn.tcp.clone();
        let mut stream = LineBufferedWriter::new(tcp.clone());
        let writer = IrcWriter::new(tx);

        // Start with identifying
        writer.identify(self.info.nick, self.info.descr);

        // Loop and handle in and output events.
        // Quit is a special case to allow us to close the program.
        for x in rx.iter() {
            match x {
                Output(ref s) => {
                    self.handle_write(s, &mut stream);
                },
                Received(ref s) => {
                    self.handle_received(s, &writer);
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

//struct IrcPrivMsg {
    //orig: String
    //prefix: String,
    //channel: String,
    //msg: String,
//}

// Commands to the bot
//struct IrcCmdMsg {
    //orig: String,
    //prefix: String,
    //channel: String,
    //cmd: String,
    //args: String,
//}

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

    // FIXME correct tests
    // Test callbacks
    //#[test]
    //fn ping() {
        //test_cb_match(super::ping, "PING :423131321", "PONG :423131321");
        //test_cb_none(super::ping, "JOIN :asdf");
    //}

    // IRC message parsing test functions
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

    // Raw callback test functions
    fn test_cb_match(f: |String| -> Option<String>, s: &str, expected: &str) {
        match f(s.to_string()) {
            Some(got) => assert_eq!(got, expected.to_string()),
            None => fail!("None"),
        }
    }

    fn test_cb_none(f: |String| -> Option<String>, s: &str) {
        match f(s.to_string()) {
            Some(_) => fail!("Some"),
            None => (),
        }
    }
}

