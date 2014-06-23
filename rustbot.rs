#![allow(dead_code)]
#![feature(globs)]
#![feature(macro_rules)]

// For regex usage
#![feature(phase)]
#[phase(plugin)]
extern crate regex_macros;
extern crate regex;

extern crate core;

use std::*;
use std::io::*;

use irc::*;
use irc::info::*;
use irc::msg::*;
use irc::privmsg::*;
use irc::connection::ConnectionEvent;
use irc::config::IrcConfig;
use irc::writer::IrcWriter;
use irc::command::{Command, IrcCommand};
mod irc;

static CMD_PREFIX: char = '.';

// Could not get this to work. Could not close over response,
//fn reply_cb<'a>(response: &'a str) -> |&IrcCommand, &IrcWriter, &BotInfo|:'a {
    //|cmd: &IrcCommand, writer: &IrcWriter, _| {
        //let r = response.to_string();
        //writer.msg_channel(cmd.channel.as_slice(), &r);
    //}
//}
// so I made a macro instead! :)
macro_rules! register_reply(
    ($irc:ident, $cmd:expr, $response:expr) => (
        $irc.register_cmd_cb($cmd, |cmd: &IrcCommand, writer: &IrcWriter, _| {
            writer.msg_channel(cmd.channel.as_slice(), &$response.to_string());
        });
    );
)

fn run_external_cmd(cmd: &str, args: &[&str]) -> String {
    let mut process = match std::io::process::Command::new(cmd).args(args).spawn() {
        Ok(p) => p,
        Err(e) => fail!("Runtime error: {}", e),
    };

    match process.stdout.get_mut_ref().read_to_end() {
        Ok(x) => {
            // Hilarious :)
            std::str::from_utf8(x.as_slice()).unwrap().to_string()
        },
        Err(e) => fail!("Read error: {}", e),
    }
}

// Can optionally send args as well in a nice manner.
// ex:
// register_external!("cmd", "/usr/bin/foo");
// register_external!("cmd", "/usr/bin/foo", "bar");
// register_external!("cmd", "/usr/bin/foo", "bar", "quux");
// and it will add args from irc.
macro_rules! register_external(
    ($irc:ident, $cmd:expr, $ext:expr) => (
        $irc.register_cmd_cb($cmd, |cmd: &IrcCommand, writer: &IrcWriter, _| {
            let response = run_external_cmd($cmd, cmd.args.as_slice());
            writer.msg_channel(cmd.channel.as_slice(), &response);
        });
    );
    ($irc:ident, $cmd:expr, $ext:expr, $($arg:tt)*) => (
        $irc.register_cmd_cb($cmd, |cmd: &IrcCommand, writer: &IrcWriter, _| {
            let mut args: Vec<&str> = vec![$($arg)*];
            let res = args.append(cmd.args.as_slice());
            let response = run_external_cmd($cmd, res.as_slice());
            writer.msg_channel(cmd.channel.as_slice(), &response);
        });
    );
)

fn main() {
    //let mut args = os::args();
    //let binary = args.shift();

    let conf = IrcConfig {
        host: "irc.quakenet.org",
        port: 6667,
        nick: "rustbot",
        descr: "https://github.com/treeman/rustbot",
        channels: vec!["#treecraft"], // Autojoin on connect

        // Input blacklist by code.
        in_blacklist: vec![
            "001", "002", "003", "004", "005",  // greetings etc
            "005",                              // supported things
            "251", "252", "253", "254", "255",  // server status, num connections etc
            "372", "375", "376",                // MOTD
            "NOTICE", "PING",                   // crap?
        ],

        // Output is blacklisted with regexes, as they lack structure.
        out_blacklist: vec![regex!(r"^PONG")],
        cmd_prefix: CMD_PREFIX,
    };

    let mut irc = Irc::connect(conf);

    // Directly hook into internal channel.
    irc.register_tx_proc(stdin_reader);

    // Utter a friendly greeting when joining
    irc.register_code_cb("JOIN", |msg: &IrcMsg, writer: &IrcWriter, info: &BotInfo| {
        writer.msg_channel(msg.param.as_slice(),
                           &format!("The Mighty {} has arrived!", info.nick));
    });

    // A simple way to be friendly.
    // TODO regex -> response macro?
    irc.register_privmsg_cb(|msg: &IrcPrivMsg, writer: &IrcWriter, _| {
        let re = regex!(r"^[Hh]ello[!.]*");
        if re.is_match(msg.txt.as_slice()) {
            writer.msg_channel(msg.channel.as_slice(),
                               &format!("Hello {}", msg.sender_nick));
        }
    });

    // Simple things.
    register_reply!(irc, "about", "I'm an irc bot written in rust as a learning experience.");
    register_reply!(irc, "src", "https://github.com/treeman/rustbot");
    register_reply!(irc, "botsnack", ":)");
    register_reply!(irc, "status", "Status: 418 I'm a teapot");

    // External scripts
    register_external!(irc, "ticker", "ticker");
    register_external!(irc, "nextep", "nextep", "--short");

    irc.run();
}

// If we shall continue the stdin loop or not.
enum StdinControl {
    Quit,
    Continue
}

// We can do some rudimentary things from the commandline.
fn stdin_cmd(cmd: &Command, writer: &IrcWriter) -> StdinControl {
    match cmd.name {
        "quit" => {
            writer.quit("Gone for repairs");
            return Quit;
        },
        "echo" => {
            let rest = cmd.args.connect(" ");
            writer.write_line(rest);
        },
        "say" => {
            if cmd.args.len() > 1 {
                let chan = cmd.args.get(0);
                let rest = cmd.args.slice_from(1).connect(" ");
                writer.msg_channel(*chan, &rest);
            }
            else {
                // <receiver> can be either a channel or a user nick
                println!("Usage: .say <receiver> text to send");
            }
        },
        _ => (),
    }
    Continue // Don't quit by default
}

// Read input from stdin.
fn stdin_reader(tx: Sender<ConnectionEvent>) {
    let writer = IrcWriter::new(tx);

    println!("Spawning stdin reader");
    for line in io::stdin().lines() {
        // FIXME prettier...
        let s : String = line.unwrap();
        let x = s.as_slice().trim();
        println!("stdin: {}", x);

        match Command::new(x, CMD_PREFIX) {
            Some(cmd) => {
                match stdin_cmd(&cmd, &writer) {
                    Quit => break,
                    _ => (),
                }
            },
            None => (),
        }
    }
    println!("Quitting stdin reader");
}

