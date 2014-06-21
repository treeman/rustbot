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
use irc::msg::*;
use irc::config::*;
use irc::writer::*;
mod irc;

// Read input from stdin.
fn spawn_stdin_reader(writer: IrcWriter) {
    println!("Spawning stdin reader");
    spawn(proc() {
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
    })
}

fn main() {
    //let mut args = os::args();
    //let binary = args.shift();

    let conf = IrcConfig {
        host: "irc.quakenet.org",
        port: 6667,
        nick: "rustbot",
        descr: "https://github.com/treeman/rustbot",
        blacklist: vec![
            "001", "002", "003", "004", "005",  // greetings etc
            "005",                              // supported things
            "251", "252", "253", "254", "255",  // server status, num connections etc
            "372", "375", "376",                // MOTD
            "NOTICE",                           // crap?
        ],
    };

    let channels = vec!["#treecraft"];

    // FIXME better interface? Or jus keep?
    let mut irc = Irc::connect(conf);
    irc.register_code_cb("004",
        |_: &IrcMsg, writer: &IrcWriter| {
            for chan in channels.iter() {
                writer.join(*chan);
            }
        });

    // FIXME make this work (need a writer!)
    //spawn_stdin_reader(irc.writer());
    irc.run();
}

