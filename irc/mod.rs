use std::io::*;
use regex::*;
use std::collections::hashmap::HashSet;
use std::collections::hashmap::HashMap;

use irc::config::*;
use irc::connection::*;
use irc::msg::IrcMsg;
use irc::privmsg::IrcPrivMsg;
use irc::writer::*;
use irc::info::BotInfo;
use irc::command::*;

pub mod config;
pub mod connection;
pub mod writer;
pub mod msg;
pub mod privmsg;
pub mod info;
pub mod command;

pub struct Irc<'a> {
    // Connections to irc server and over internal channel.
    conn: ServerConnection,

    // General config.
    info: BotInfo<'a>,
    // String to avoid lifetime issues :)
    in_blacklist: HashSet<String>,
    out_blacklist: Vec<Regex>,

    // Callbacks at received events.
    raw_cb: Vec<|&str, &IrcWriter, &BotInfo|:'a>,

    // This is a workaround for a multimap.
    code_cb: HashMap<String, Vec<|&IrcMsg, &IrcWriter, &BotInfo|:'a>>,

    // Callbacks for PRIVMSG.
    privmsg_cb: Vec<|&IrcPrivMsg, &IrcWriter, &BotInfo|:'a>,

    // Command callbacks. Multimap.
    cmd_cb: HashMap<String, Vec<|&IrcCommand, &IrcWriter, &BotInfo|:'a>>,

    // We can register external functions to be spawned during runtime.
    // Workaround as I couldn't get Irc to hold a valid tx we can return.
    // The problem is what to do with the rx.
    spawn_funcs: Vec<fn(Sender<ConnectionEvent>)>,
}

impl<'a> Irc<'a> {
    // Create a new irc instance and connect to the server, but don't act on it.
    pub fn connect<'b>(conf: IrcConfig<'b>) -> Irc<'b> {

        // Couldn't there be a nicer way to do this?
        let mut in_blacklist = HashSet::new();
        for x in conf.in_blacklist.iter() {
            in_blacklist.insert(x.to_string());
        }

        let mut irc = Irc {
            conn: ServerConnection::new(conf.host, conf.port),

            info: BotInfo::new(&conf),
            in_blacklist: in_blacklist,
            out_blacklist: conf.out_blacklist,

            raw_cb: Vec::new(),
            code_cb: HashMap::new(),
            privmsg_cb: Vec::new(),
            cmd_cb: HashMap::new(),
            spawn_funcs: Vec::new(),
        };

        irc.init_callbacks();
        irc
    }

    // Register a callback for a specific command.
    pub fn register_cmd_cb(&mut self, cmd: &str,
                           cb: |&IrcCommand, &IrcWriter, &BotInfo|:'a)
    {
        let c = cmd.to_string();
        if !self.cmd_cb.contains_key(&c) {
            self.cmd_cb.insert(c.clone(), Vec::new());
        }
        let cbs = self.cmd_cb.get_mut(&c);
        cbs.push(cb);
    }

    // Register a callback for a specific irc msg code.
    pub fn register_code_cb(&mut self, code: &str, cb: |&IrcMsg, &IrcWriter, &BotInfo|:'a)
    {
        let c = code.to_string();
        if !self.code_cb.contains_key(&c) {
            self.code_cb.insert(c.clone(), Vec::new());
        }
        let cbs = self.code_cb.get_mut(&c);
        cbs.push(cb);
    }

    // Register a callback for a PRIVMSG.
    pub fn register_privmsg_cb(&mut self,
                               cb: |&IrcPrivMsg, &IrcWriter, &BotInfo|:'a)
    {
        self.privmsg_cb.push(cb);
    }

    fn init_callbacks(&mut self) {
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

    // Called when we see a PRIVMSG.
    fn handle_priv_msg(&mut self, msg: &IrcPrivMsg, writer: &IrcWriter) {
        for cb in self.privmsg_cb.mut_iter() {
            (*cb)(msg, writer, &self.info);
        }
    }

    // Called when we receive a command from irc.
    fn handle_cmd(&mut self, cmd: &IrcCommand, writer: &IrcWriter) {
        // Irc cmd callbacks.
        let c = cmd.name.to_string();
        if self.cmd_cb.contains_key(&c) {
            let cbs = self.cmd_cb.get_mut(&c);
            for cb in cbs.mut_iter() {
                (*cb)(cmd, writer, &self.info);
            }
        }
    }

    // Called when we have a properly formatted irc message.
    fn handle_msg(&mut self, msg: &IrcMsg, writer: &IrcWriter) {
        // Print received message if it's not blacklisted.
        let code = msg.code.clone();
        if !self.in_blacklist.contains(&code) {
            println!("< {}", msg.orig);
        }

        // Irc msg callbacks.
        let c = msg.code.clone();
        if self.code_cb.contains_key(&c) {
            let cbs = self.code_cb.get_mut(&c);
            for cb in cbs.mut_iter() {
                (*cb)(msg, writer, &self.info);
            }
        }

        // Should be able to avoid nesting like this.
        match IrcPrivMsg::new(msg) {
            Some(msg) => {
                self.handle_priv_msg(&msg, writer);

                match IrcCommand::new(&msg, self.info.cmd_prefix) {
                    Some(cmd) => self.handle_cmd(&cmd, writer),
                    _ => (),
                }
            },
            _ => (),
        }
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
        let s = s.as_slice();
        for re in self.out_blacklist.iter() {
            if !re.is_match(s) {
                println!("> {}", s);
            }
        }
        write_line(stream, s);
    }

    // Run irc client and block until done.
    pub fn run(&mut self) {
        let (tx, rx) = channel();

        // Spawn reader which reads from our tcp.
        self.spawn_reader(tx.clone());

        // Spawn registered functions with a tx.
        for f in self.spawn_funcs.mut_iter() {
            let fun = *f; // Owner workaround, &fn isn't sendable but fn is.
            let tx = tx.clone(); // Create a tx for proc to own.
            spawn(proc() {
                fun(tx);
            });
        }
        self.run_handler(tx.clone(), rx);
    }

    // Expose internal tx channel through these callbacks.
    // Workaround as I couldn't make irc hold tx (and the problematic rx).
    pub fn register_tx_proc(&mut self, f: fn(Sender<ConnectionEvent>)) {
        self.spawn_funcs.push(f);
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
        println!("Running event handler");

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
    // FIXME how to test callbacks?
    // Hook into rx/tx?
}

