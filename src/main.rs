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
        Mode::Help
    } else if matches.opt_present("version") {
        Mode::Version
    } else {
        Mode::Run
    };

    let config = match matches.opt_str("c") {
        Some(c) => c,
        None => "config.json".to_string()
    };

    match mode {
        Mode::Help => help(progname[], usage[]),
        Mode::Version => version(),
        Mode::Run => run(config)
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
    // The problem lies with stack closures which must live until irc.run()
    // so they need to be in the same function.

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

    // .schema gives us a schedule for liu
    irc.register_cmd_cb("schema", |cmd: &IrcCommand, writer: &IrcWriter, _| {
        // TODO pass ircwriter...
        let ans = find_schema(&cmd.args);
        writer.msg(cmd.channel[], ans[]);
    });

    irc.run();
}

fn find_schema(args: &Vec<&str>) -> String {
    println!("args: `{}`", args);

    // Arguments:
    // --tomorrow -t (list whole day tomorrow)

    let base = "https://se.timeedit.net/web/liu/db1/schema";

    let s = util::join(args, " ");
    let from = time::now();
    let to = time::at(from.to_timespec() + Duration::weeks(1));

    let types = timeedit::multi_search(s[], base);

    let mut res = String::new();
    if types.is_empty() {
        return "So sorry, no match found.".to_string();
    } else {
        res.push_str("Schedule for: ");

        let codes = util::join(&types.iter().map(|x| x.code[]).collect(), ", ");
        res.push_str(codes[]);

        let events = timeedit::schedule(types, from, to, base);
        //res.push_str(format!("\nFound {} events this week", events.len())[]);
        //res.push_str(format!("\nNext event: {}", events[0].fmt_full())[]);

        // If there are things today, list them all
        let today = timeedit::filter_today(events.clone());
        if !today.is_empty() {
            for event in today.iter() {
                res.push_str(format!("\n{}", event.fmt_time_only())[]);
            }
        // Otherwise just print when the next is
        } else {
            let events = timeedit::filter_upcoming(events);

            if events.is_empty() {
                res.push_str("\nYou're free!");
            } else {
                res.push_str(format!("\nNext: {}", events[0].fmt_pretty())[]);
            }
        }
    }
    res
}

// FIXME simplify...
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
            return StdinControl::Quit;
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
    StdinControl::Continue // Don't quit by default
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
                    StdinControl::Quit => break,
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

