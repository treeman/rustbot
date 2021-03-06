#![macro_escape]

use std::io::*;
use std::time::Duration;

use irc::config::*;
use irc::connection::*;
use irc::msg::IrcMsg;
use irc::privmsg::IrcPrivMsg;
use irc::writer::*;
use irc::info::BotInfo;
use irc::command::*;
use irc::data::*;
use irc::Plugin;

pub struct Irc<'a> {
    // Connections to irc server and over internal channel.
    conn: ServerConnection,

    // All irc data.
    data: IrcData<'a>,
}

impl<'a> Irc<'a> {
    // Create a new irc instance and connect to the server, but don't act on it.
    pub fn connect<'b>(conf: IrcConfig<'b>) -> Irc<'b> {

        // Split into connection and data,
        // so we can read from an rx and still
        // iterate over callbacks, which needs to be mut_iter
        let mut irc = Irc {
            conn: ServerConnection::new(conf.host, conf.port),
            data: IrcData::new(conf),
        };

        irc.init_callbacks();
        irc
    }

    // Register a callback for a specific command.
    pub fn register_cmd_cb(&mut self, cmd: &str,
                           cb: |&IrcCommand, &IrcWriter, &BotInfo|:'a)
    {
        let c = cmd.to_string();
        if !self.data.cmd_cb.contains_key(&c) {
            self.data.cmd_cb.insert(c.clone(), Vec::new());
        }
        let cbs = self.data.cmd_cb.get_mut(&c).unwrap();
        cbs.push(cb);
    }

    // Register a callback for a specific irc msg code.
    pub fn register_code_cb(&mut self, code: &str, cb: |&IrcMsg, &IrcWriter, &BotInfo|:'a) {
        let c = code.to_string();
        if !self.data.code_cb.contains_key(&c) {
            self.data.code_cb.insert(c.clone(), Vec::new());
        }
        let cbs = self.data.code_cb.get_mut(&c).unwrap();
        cbs.push(cb);
    }

    // Register a callback for a PRIVMSG.
    pub fn register_privmsg_cb(&mut self, cb: |&IrcPrivMsg, &IrcWriter, &BotInfo|:'a) {
        self.data.privmsg_cb.push(cb);
    }

    pub fn register_plugin(&mut self, plugin: Box<Plugin + 'a>) {
        self.data.plugins.push(plugin);
    }

    fn init_callbacks(&mut self) {
        self.register_code_cb("PING", |msg: &IrcMsg, writer: &IrcWriter, _| {
            writer.output(format!("PONG {}", msg.param));
        });

        // Key on 004, should be fine as it's usually in the beginning I believe?
        self.register_code_cb("004", |_, writer: &IrcWriter, info: &BotInfo| {
            for chan in info.channels.iter() {
                writer.join(*chan);
            }
        });
    }

    // Run irc client and block until done.
    pub fn run(self) {
        // Spawn reader which reads from our tcp.
        self.spawn_reader(self.conn.tx.clone());

        let tx = self.conn.tx.clone();
        self.run_handler(tx);
    }

    // Return a handle we can write through irc with.
    pub fn writer(&self) -> IrcWriter {
        IrcWriter::new(self.conn.tx.clone())
    }

    // Spawn a proc reader which listens to incoming messages from irc.
    fn spawn_reader(&self, tx: Sender<ConnectionEvent>) {
        println!("Spawning irc reader");
        let tcp = self.conn.tcp.clone(); // Workaround to avoid irc capture
        spawn(proc() {
            let mut reader = BufferedReader::new(tcp);
            let mut attempt = 0u;
            loop {
                match reader.read_line() {
                    Ok(x) => {
                        tx.send(ConnectionEvent::Received(x));
                        if attempt > 0 {
                            println!("Attempt {} successful!", attempt);
                        }
                        attempt = 0;
                    },

                    Err(e) => {
                        println!("Error! {}", e);
                        attempt += 1;
                    },
                }

                // If we fail, only attempt it 5 times.
                if attempt == 5 {
                    break;
                // Use a 5 second delay between possible attempts.
                } else if attempt > 0 {
                    println!("Waiting 5 seconds before next attempt...");
                    timer::sleep(Duration::seconds(5));
                }
            }
            println!("Quitting irc reader");
        });
    }

    // Run event handler. Will block.
    fn run_handler(self, tx: Sender<ConnectionEvent>) {
        println!("Running event handler");

        let tcp = self.conn.tcp.clone();
        let mut stream = LineBufferedWriter::new(tcp.clone());
        let writer = IrcWriter::new(tx);

        // Start with identifying
        writer.identify(self.data.info.nick, self.data.info.descr);

        let conn = self.conn;
        let mut data = self.data;

        // Loop and handle in and output events.
        // Quit is a special case to allow us to close the program.
        for x in conn.rx.iter() {
            match x {
                ConnectionEvent::Output(ref s) => {
                    data.handle_write(s, &mut stream);
                },
                ConnectionEvent::Received(ref s) => {
                    data.handle_received(s, &writer);
                },
                ConnectionEvent::Quit => {
                    break;
                },
            }
        }
        conn.close();
        println!("Exiting irc writer");
    }
}

// Could not get this to work. Could not close over response,
//fn reply_cb<'a>(response: &'a str) -> |&IrcCommand, &IrcWriter, &BotInfo|:'a {
    //|cmd: &IrcCommand, writer: &IrcWriter, _| {
        //let r = response.to_string();
        //writer.msg_channel(cmd.channel[], &r);
    //}
//}
// so I made a macro instead! :)
//
// Simple way of registering a simple
// .cmd -> response
// ex: register_reply!(irc, "cheese", ":D");
#[macro_export]
macro_rules! register_reply(
    ($irc:ident, $cmd:expr, $response:expr) => (
        $irc.register_cmd_cb($cmd, |cmd: &IrcCommand, writer: &IrcWriter, _| {
            writer.msg(cmd.channel[], $response);
        });
    );
)

// Can optionally send args as well in a nice manner.
// ex:
// register_external!("cmd", "/usr/bin/foo");
// register_external!("cmd", "/usr/bin/foo", "bar");
// register_external!("cmd", "/usr/bin/foo", "bar", "quux");
// and it will add args from irc.
#[macro_export]
macro_rules! register_external(
    ($irc:ident, $cmd:expr, $ext:expr) => (
        $irc.register_cmd_cb($cmd, |cmd: &IrcCommand, writer: &IrcWriter, _| {
            let response = run_external_cmd($cmd, cmd.args[]);
            writer.msg_channel(cmd.channel[], &response);
        });
    );
    ($irc:ident, $cmd:expr, $ext:expr, $($arg:tt)*) => (
        $irc.register_cmd_cb($cmd, |cmd: &IrcCommand, writer: &IrcWriter, _| {
            let mut args: Vec<&str> = vec![$($arg)*];
            args.push_all(cmd.args[]);
            let response = util::run_external_cmd($cmd, args[]);
            writer.msg(cmd.channel, response[]);
        });
    );
)

#[cfg(test)]
mod tests {
    // FIXME how to test callbacks?
    // Hook into rx/tx?
}

