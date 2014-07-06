#![allow(dead_code)]
#![feature(globs)]
#![feature(macro_rules)]

// For regex usage
#![feature(phase)]
#[phase(plugin)]
extern crate regex_macros;
extern crate regex;

extern crate core;
extern crate time;

use std::*;
use std::io::*;
use std::io::Timer;
use time::*;

use irc::*;

mod irc;
mod util;

static CMD_PREFIX: char = '.';

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
            "001", "002", "003", "004",         // greetings etc
            "005",                              // supported things
            "251", "252", "253", "254", "255",  // server status, num connections etc
            "372", "375", "376",                // MOTD
            "PING",                             // crap?
        ],

        // Output is blacklisted with regexes, as they lack structure.
        out_blacklist: vec![regex!(r"^PONG")],
        cmd_prefix: CMD_PREFIX,
    };

    let mut irc = Irc::connect(conf);

    // TODO refactor callbacks etc...

    // Make it so we can read commands from stdin.
    let writer = irc.writer();
    spawn(proc() {
        stdin_reader(writer);
    });

    let writer = irc.writer();
    spawn(proc() {
        reminder(writer);
    });

    // Utter a friendly greeting when joining
    irc.register_code_cb("JOIN", |msg: &IrcMsg, writer: &IrcWriter, info: &BotInfo| {
        if msg.prefix.as_slice().contains(info.nick) {
            writer.msg(msg.param.as_slice(),
                    format!("The Mighty {} has arrived!", info.nick).as_slice());
        }
    });

    // A simple way to be friendly.
    // TODO regex -> response macro?
    irc.register_privmsg_cb(|msg: &IrcPrivMsg, writer: &IrcWriter, _| {
        let re = regex!(r"^[Hh]ello[!.]*");
        if re.is_match(msg.txt.as_slice()) {
            writer.msg(msg.channel.as_slice(),
                       format!("Hello {}", msg.sender_nick).as_slice());
        }
    });

    // Simple help
    let help_txt = "I'm a simple irc bot. Prefix commands with .";
    irc.register_privmsg_cb(|msg: &IrcPrivMsg, writer: &IrcWriter, _| {
        let txt = msg.txt.as_slice().trim();
        if txt == "help" {
            writer.msg(msg.channel.as_slice(), help_txt);
        }
    });

    // Simple things.
    register_reply!(irc, "about", "I'm an irc bot written in rust as a learning experience.");
    register_reply!(irc, "src", "https://github.com/treeman/rustbot");
    register_reply!(irc, "botsnack", ":)");
    register_reply!(irc, "status", "Status: 418 I'm a teapot");
    register_reply!(irc, "help", help_txt);

    // External scripts
    register_external!(irc, "nextep", "nextep", "--short");

    // .uptime return the runtime of our bot
    let start = now();
    irc.register_cmd_cb("uptime", |cmd: &IrcCommand, writer: &IrcWriter, _| {
        let at = now();
        let dt = at.to_timespec().sec - start.to_timespec().sec;
        writer.msg(cmd.channel.as_slice(), format!("I've been alive {}", format(dt)).as_slice());
    });

    irc.run();
}

// 12 days 2 hours 3 minutes 48 seconds
fn format(sec: i64) -> String {
    let min: i64 = sec / 60;
    let hours: i64 = min / 60;
    let days: i64 = hours / 24;

    if days > 0 {
        format!("{} days {} hours {} minutes {} seconds", days, hours, min, sec)
    } else if hours > 0 {
        format!("{} hours {} minutes {} seconds", hours, min, sec)
    } else if min > 0 {
        format!("{} minutes {} seconds", min, sec)
    } else {
        format!("{} seconds", sec)
    }
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
            writer.output(rest);
        },
        "say" => {
            if cmd.args.len() > 1 {
                let chan = cmd.args.get(0);
                let rest = cmd.args.slice_from(1).connect(" ");
                writer.msg(*chan, rest.as_slice());
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
fn stdin_reader(writer: IrcWriter) {
    println!("Spawning stdin reader");
    for line in io::stdin().lines() {
        // FIXME prettier...
        let s : String = line.unwrap();
        let x = s.as_slice().trim();

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

// Send a friendly reminder!
fn reminder(writer: IrcWriter) {
    let mut timer = Timer::new().unwrap();
    let mut sent = false;

    // Execute the loop every 10 minutes
    let periodic = timer.periodic(1000 * 60 * 10);
    loop {
        periodic.recv();

        // Key on every 23:th hour
        let curr = now();

        if curr.tm_hour == 23 {
            if !sent {
                writer.msg("Firekite", "You need to kill things in habitrpg!");
                sent = true;
            }
        } else {
            sent = false;
        }
    }
}

