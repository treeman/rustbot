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
        channels: vec!["#treecraft"],
        blacklist: vec![
            "001", "002", "003", "004", "005",  // greetings etc
            "005",                              // supported things
            "251", "252", "253", "254", "255",  // server status, num connections etc
            "372", "375", "376",                // MOTD
            "NOTICE",                           // crap?
        ],
    };

    let mut irc = Irc::connect(conf);

    // Directly hook into internal channel.
    irc.register_tx_proc(stdin_reader);

    irc.run();
}

