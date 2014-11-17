#![allow(dead_code)]
#![feature(globs)]
#![feature(macro_rules)]
#![feature(slicing_syntax)]

// For regex usage
#![feature(phase)]
#[phase(plugin)]
extern crate regex_macros;
extern crate regex;
extern crate serialize;
extern crate getopts;

extern crate core;
extern crate time;

use std::os;
use std::io;
use std::io::Timer;
use std::time::Duration;
use regex::Regex;

use getopts::{
    optopt,
    optflag,
    getopts,
    usage
};

use irc::Irc;
use irc::info::*;
use irc::msg::*;
use irc::privmsg::*;
use irc::config::{IrcConfig, JsonConfig};
use irc::writer::IrcWriter;
use irc::command::{Command, IrcCommand};

mod irc;
mod util;

static CMD_PREFIX: char = '.';

fn main() {
    let args = os::args();

    let opts = [
        optopt("c", "config", "Specify config file, default: config.json", "CFILE"),
        optflag("v", "version", "Output version information and exit"),
        optflag("h", "help", "display this help and exit")
    ];

    let matches = match getopts(args.tail(), opts) {
        Ok(m) => m,
        Err(e) => panic!("{}", e)
    };

    let progname = args[0].clone();
    let usage = usage("Starts rustbot, an IRC bot written in rust.", opts);

    let mode = if matches.opt_present("help") {
        Help
    } else if matches.opt_present("version") {
        Version
    } else {
        Run
    };

    let config = match matches.opt_str("c") {
        Some(c) => c,
        None => "config.json".to_string()
    };

    match mode {
        Help => help(progname[], usage[]),
        Version => version(),
        Run => run(config)
    }
}

fn run(config: String) {
    let jconf = JsonConfig::new(config);

    let conf = IrcConfig {
        host: jconf.host[],
        port: jconf.port,
        nick: jconf.nick[],
        descr: jconf.descr[],
        channels: jconf.channels.iter().map(|x| x[]).collect(), // Autojoin on connect

        // Input blacklist by code.
        in_blacklist: jconf.in_blacklist.iter().map(|x| x[]).collect(),

        // Output is blacklisted with regexes, as they lack structure.
        out_blacklist: jconf.out_blacklist.iter().map(
            |x| {
                match Regex::new(x[]) {
                    Ok(re) => re,
                    Err(err) => panic!("{}", err),
                }
            }).collect(),
        cmd_prefix: jconf.cmd_prefix,
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
        if msg.prefix[].contains(info.nick) {
            writer.msg(msg.param[],
                    format!("The Mighty {} has arrived!", info.nick)[]);
        }
    });

    // A simple way to be friendly.
    // TODO regex -> response macro?
    irc.register_privmsg_cb(|msg: &IrcPrivMsg, writer: &IrcWriter, _| {
        let re = regex!(r"^[Hh]ello[!.]*");
        if re.is_match(msg.txt[]) {
            writer.msg(msg.channel[],
                       format!("Hello {}", msg.sender_nick)[]);
        }
    });

    // Simple help
    let help_txt = "I'm a simple irc bot. Prefix commands with .";
    irc.register_privmsg_cb(|msg: &IrcPrivMsg, writer: &IrcWriter, _| {
        let txt = msg.txt[].trim();
        if txt == "help" {
            writer.msg(msg.channel[], help_txt);
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
    let start = time::now();
    irc.register_cmd_cb("uptime", |cmd: &IrcCommand, writer: &IrcWriter, _| {
        let at = time::now();
        let dt = at.to_timespec().sec - start.to_timespec().sec;
        writer.msg(cmd.channel[], format!("I've been alive {}", format(dt))[]);
    });

    irc.run();
}

// 12 days 2 hours 3 minutes 48 seconds
fn format(mut sec: i64) -> String {
    let mut min: i64 = sec / 60;
    let mut hours: i64 = min / 60;
    let days: i64 = hours / 24;

    if sec > 0 {
        sec = sec - min * 60;
    }
    if hours > 0 {
        min = min - hours * 60;
    }
    if days > 0 {
        hours = hours - days * 24;
    }

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
                let chan = cmd.args[0];
                let rest = cmd.args.slice_from(1).connect(" ");
                writer.msg(chan, rest[]);
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
        let x = s[].trim();

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

fn help(progname: &str, usage: &str) {
    println!("Usage: {:s} [OPTION]", progname);
    io::stdio::println(usage);
}

// FIXME Load version from Cargo.toml
fn version() {
    println!("rustbot 0.0.1");
}

enum Mode {
    Help,
    Version,
    Run
}

// Send a friendly reminder!
fn reminder(writer: IrcWriter) {
    let mut timer = Timer::new().unwrap();
    let mut sent = false;

    let periodic = timer.periodic(Duration::minutes(10));
    loop {
        periodic.recv();

        // Key on every 23:th hour
        let curr = time::now();

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

