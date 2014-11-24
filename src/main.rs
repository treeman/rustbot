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
extern crate timeedit;

use std::os;
use std::io::{ mod, Timer };
use std::time::Duration;
use regex::Regex;

use getopts::{
    optopt,
    optflag,
    getopts,
    usage
};

use irc::*;

mod irc;
mod util;
mod plugins;
mod stdin;

static CMD_PREFIX: char = '.';

fn main() {
    let args = os::args();

    let opts = [
        optopt("c", "config", "Specify config file, default: config.json", "CFILE"),
        optflag("v", "version", "Output version information and exit"),
        optflag("h", "help", "display this help and exit")
    ];

    let matches = match getopts(args.tail(), &opts) {
        Ok(m) => m,
        Err(e) => panic!("{}", e)
    };

    let progname = args[0].clone();
    let usage = usage("Starts rustbot, an IRC bot written in rust.", &opts);

    let config = match matches.opt_str("c") {
        Some(c) => c,
        None => "config.json".to_string()
    };

    if matches.opt_present("help") {
        help(progname[], usage[])
    } else if matches.opt_present("version") {
        version()
    } else {
        run(config)
    };
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
    // The problem lies with stack closures which must live until irc.run()
    // so they need to be in the same function.

    // Make it so we can read commands from stdin.
    let writer = irc.writer();
    spawn(proc() {
        stdin::reader(writer);
    });

    // Spawn a periodically called function
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

    // Register some heavier plugins
    plugins::register(&mut irc);

    irc.run();
}

fn help(progname: &str, usage: &str) {
    println!("Usage: {} [OPTION]", progname);
    io::stdio::println(usage);
}

// FIXME Load version from Cargo.toml
fn version() {
    println!("rustbot 0.0.1");
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

