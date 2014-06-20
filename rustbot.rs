#![feature(globs)]

use std::*;
use std::io::*;
use irc::*;
use writer::*;

mod connection;
mod writer;
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
                writer.send_quit("Gone for repairs".to_string());
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
        channels: vec!["#treecraft"],
        nick: "rustbot",
        descr: "https://github.com/treeman/rustbot"
    };

    let mut irc = Irc::connect(conf);
    spawn_stdin_reader(irc.writer());
    irc.run();
}

