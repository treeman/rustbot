#![allow(dead_code)]
#![feature(globs)]

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
use irc::command::Command;
mod irc;

static CMD_KEY: char = '.';

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

        match Command::new(x, CMD_KEY) {
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
    };

    let mut irc = Irc::connect(conf);

    // Directly hook into internal channel.
    irc.register_tx_proc(stdin_reader);

    // Utter a friendly greeting when joining
    irc.register_code_cb("JOIN", |msg: &IrcMsg, writer: &IrcWriter, info: &BotInfo| {
        let chan = msg.param.clone();
        writer.msg_channel(chan.as_slice(), &format!("The Mighty {} has arrived!", info.nick));
    });

    // A simple way to be friendly.
    irc.register_privmsg_cb(|msg: &IrcPrivMsg, writer: &IrcWriter, _| {
        let re = regex!(r"^[Hh]ello[!.]*");
        if re.is_match(msg.msg.as_slice()) {
            writer.msg_channel(msg.channel.as_slice(), &format!("Hello {}", msg.sender_nick));
        }
    });

    irc.run();
}

