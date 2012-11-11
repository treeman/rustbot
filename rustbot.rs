extern mod std(vers = "0.5");

use std::*;
use getopts::*;

use io::println;
use io::*;

use ip = net::ip;
use socket = net::tcp;

use task;
use uv::iotask;
use uv::iotask::iotask;

use core::str;
use core::str::*;

use core::vec;
use core::vec::*;

use irc;
use irc::*;

fn usage(binary: &str) {
    io::println(fmt!("Usage: %s [options]\n", binary) +
            "
Options:

    -h --help           Show this helpful screen
");
}

fn handle(irc: &Irc, m: &IrcMsg) {
    match m.code {
        // Hook channel join here. Made sense at the time?
        ~"004" => {
            join(irc, "#madeoftree");
        }
        ~"JOIN" => {
            privmsg(irc, m.param, "yoyo the mighty rustbot has arrived!");
        }
        ~"PRIVMSG" => {
            let msg = parse_privmsg(m.param);

            if msg.msg.starts_with("hello") {
                privmsg(irc, msg.channel, "hello there mister!");
            }
            else if msg.msg.starts_with(".") {
                let split = split_char(msg.msg.slice(1, msg.msg.len()), ' ');
                let cmd = split.head();
                let args = split.tail();
                let rest = foldl(~"", args, |a,e| a + *e + " ").trim();

                match cmd {
                    ~"help" => {
                        privmsg(irc, msg.channel, "no help for ya!");
                    }
                    ~"say" => {
                        if rest != ~"" {
                            privmsg(irc, msg.channel, rest);
                        }
                    }
                    ~"insult" => {
                        if rest != ~"" {
                            privmsg(irc, msg.channel, fmt!("%s thinks rust is iron oxides.", rest));
                        }
                    }
                    ~"compliment" => {
                        if rest != ~"" {
                            privmsg(irc, msg.channel, fmt!("%s is friends with rust.", rest));
                        }
                    }
                    _ => (),
                }
            }
        }
        _ => (),
    }
}

fn main() {
    let mut args = os::args();
    let binary = args.shift();

    let opts = ~[
        getopts::optflag("h"),
        getopts::optflag("help"),
    ];

    let matches = match getopts(args, opts) {
        Ok(m) => copy m,
        Err(f) => {
            io::println(fmt!("Error: %s\n", getopts::fail_str(copy f)));
            usage(binary);
            return;
        }
    };

    if opt_present(copy matches, "h") || opt_present(copy matches, "help") {
        usage(binary);
        return;
    }

    let server = ~"irc.quakenet.org";
    let port = 6667u;
    let nickname = ~"rustbot";
    let username = ~"rustbot";
    let realname = ~"I'm a bot written in the wonderful rust language, see rust-lang.org!";

    let irc = connect(server, port);

    identify(irc, nickname, username, realname);

    run(irc, handle);
}

