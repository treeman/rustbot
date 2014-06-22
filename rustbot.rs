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
mod irc;

// Read input from stdin.
fn stdin_reader(tx: Sender<ConnectionEvent>) {
    let writer = IrcWriter::new(tx);

    println!("Spawning stdin reader");
    for line in io::stdin().lines() {
        // FIXME prettier...
        let s : String = line.unwrap();
        let x = s.as_slice().trim();
        println!("stdin: {}", x);

        if x == ".quit" {
            writer.quit("Gone for repairs");
            break;
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

    irc.register_code_cb("JOIN", |msg: &IrcMsg, writer: &IrcWriter, info: &BotInfo| {
        let chan = msg.param.clone();
        writer.msg_channel(&chan, format!("The Mighty {} has arrived!", info.nick));
    });

    irc.register_privmsg_cb(|msg: &IrcPrivMsg, writer: &IrcWriter, _| {
        let re = regex!(r"^[Hh]ello[!.]*");
        if re.is_match(msg.msg.as_slice()) {
            writer.msg_channel(&msg.channel, format!("Hello {}", msg.sender_nick));
        }
    });

    irc.run();
}

