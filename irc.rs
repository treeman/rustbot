// For awesome regex usage
#![feature(phase)]
#[phase(plugin)]
extern crate regex_macros;
extern crate regex;

use std::io::*;
use connection::*;
use writer::*;

// Single server for now.
pub struct IrcConfig<'a> {
    pub host: &'a str,
    pub port: u16,
    pub channels: Vec<&'a str>,
    pub nick: &'a str,
    pub descr: &'a str,
}

// command callbacks?
// + register functionsto send to irc.
pub struct Irc {
    // Connections to irc server and over internal channel.
    conn: ServerConnection,
    tx: Sender<IrcCommand>,
    rx: Receiver<IrcCommand>,

    // Bot info.
    nick: String,
    descr: String,
}

impl Irc {
    // Create a new irc instance and connect to the server, but don't act on it.
    pub fn connect(conf: IrcConfig) -> Irc {
        let (tx, rx) = channel();
        Irc {
            conn: ServerConnection::new(conf.host, conf.port),
            tx: tx,
            rx: rx,
            nick: conf.nick.to_string(),
            descr: conf.descr.to_string(),
        }
    }

    // Construct a writer we can use to send things to irc.
    // Uses a channel transmitter with a process in the backround.
    pub fn writer(&self) -> IrcWriter {
        IrcWriter::new(self.tx.clone())
    }

    // Run irc client and block until done.
    pub fn run(&mut self) {
        self.spawn_reader();
        self.run_writer();
    }

    // Spawn a proc reader which listens to incoming messages from irc.
    fn spawn_reader(&self) {
        println!("Spawning irc reader");
        let writer = self.writer();
        let tcp = self.conn.tcp.clone(); // Workaround to avoid irc capture
        spawn(proc() {
            let mut reader = BufferedReader::new(tcp);
            loop {
                match read_line(&mut reader) {
                    Some(x) => {
                        // FIXME text handling somewhere else
                        let s = x.as_slice().trim();
                        if s.starts_with("PING") {
                            let res = s.slice(6, s.len());
                            writer.write_line(format!("PONG :{}", res));
                        }
                    }
                    None => break
                }
            }
            println!("Quitting irc reader");
        });
    }

    fn run_writer(&mut self) {
        println!("Running writer");
        let tcp = self.conn.tcp.clone();
        let mut stream = LineBufferedWriter::new(tcp.clone());
        let writer = self.writer();

        writer.identify(&self.nick, &self.descr);
        for x in self.rx.iter() {
            match x {
                Output(s) => {
                    write_line(&mut stream, s.as_slice());
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

