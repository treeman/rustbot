use std::io::*;
use regex::*;
use std::collections::HashSet;
use std::collections::HashMap;

use irc::config::*;
use irc::connection::*;
use irc::msg::IrcMsg;
use irc::privmsg::IrcPrivMsg;
use irc::writer::*;
use irc::info::BotInfo;
use irc::command::*;

// FIXME use of callbacks!

pub struct IrcData<'a> {
    // General config.
    pub info: BotInfo<'a>,

    // String to avoid lifetime issues :)
    pub in_blacklist: HashSet<String>,
    pub out_blacklist: Vec<Regex>,

    // Callbacks at received events.
    pub raw_cb: Vec<|&str, &IrcWriter, &BotInfo|:'a>,

    // This is a workaround for a multimap.
    pub code_cb: HashMap<String, Vec<|&IrcMsg, &IrcWriter, &BotInfo|:'a>>,

    // Callbacks for PRIVMSG.
    pub privmsg_cb: Vec<|&IrcPrivMsg, &IrcWriter, &BotInfo|:'a>,

    // Command callbacks. Multimap.
    pub cmd_cb: HashMap<String, Vec<|&IrcCommand, &IrcWriter, &BotInfo|:'a>>,

    // We can register external functions to be spawned during runtime.
    // Workaround as I couldn't get Irc to hold a valid tx we can return.
    // The problem is what to do with the rx.
    pub spawn_funcs: Vec<fn(Sender<ConnectionEvent>)>,
}

impl <'a> IrcData<'a> {
    pub fn new<'b>(conf: IrcConfig<'b>) -> IrcData<'b> {
        // Couldn't there be a nicer way to do this?
        let mut in_blacklist = HashSet::new();
        for x in conf.in_blacklist.iter() {
            in_blacklist.insert(x.to_string());
        }

        IrcData {
            info: BotInfo::new(&conf),
            in_blacklist: in_blacklist,
            out_blacklist: conf.out_blacklist,

            raw_cb: Vec::new(),
            code_cb: HashMap::new(),
            privmsg_cb: Vec::new(),
            cmd_cb: HashMap::new(),
            spawn_funcs: Vec::new(),
        }
    }

    // Actually write something to irc.
    pub fn handle_write(&self, s: &String, stream: &mut LineBufferedWriter<TcpStream>) {
        let s = s.as_slice();
        let mut blacklisted = false;
        for re in self.out_blacklist.iter() {
            if re.is_match(s) {
                blacklisted = true;
            }
        }
        if !blacklisted {
            println!("> {}", s);
        }
        write_line(stream, s);
    }

    // Called when we receive a response from the server.
    pub fn handle_received(&mut self, line: &String, writer: &IrcWriter) {
        // Trim away newlines and unneeded spaces.
        let s = line.as_slice().trim();

        for cb in self.raw_cb.iter() {
            //(*cb)(s, writer, &self.info);
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

    // Called when we see a PRIVMSG.
    fn handle_priv_msg(&mut self, msg: &IrcPrivMsg, writer: &IrcWriter) {
        for cb in self.privmsg_cb.iter() {
            //(*cb)(msg, writer, &self.info);
        }
    }

    // Called when we receive a command from irc.
    fn handle_cmd(&mut self, cmd: &IrcCommand, writer: &IrcWriter) {
        // Irc cmd callbacks.
        let c = cmd.name.to_string();
        if self.cmd_cb.contains_key(&c) {
            let cbs = self.cmd_cb.get_mut(&c).unwrap();
            for cb in cbs.iter() {
                //(*cb)(cmd, writer, &self.info);
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
            let cbs = self.code_cb.get_mut(&c).unwrap();
            for cb in cbs.iter() {
                //(*cb)(msg, writer, &self.info);
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
}