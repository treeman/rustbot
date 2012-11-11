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
        _ => (),
    }
}

fn register_callbacks(irc: &Irc) {
    // TODO suppress unused parameter error?
    register(~"help", irc, |m| ~"Prefix commands with a '.' and try '.cmds'");
    register(~"about", irc, |m| ~"I'm written in rust as a learning experience, try http://www.rust-lang.org!");
    register(~"insult", irc, |m| fmt!("%s thinks rust is iron oxide.", m.arg));
    register(~"compliment", irc, |m| fmt!("%s is best friends with rust.", m.arg));
    register(~"botsnack", irc, |m| ~":)");
    register(~"status", irc, |m| ~"Status: 418 I'm a teapot");
    register(~"src", irc, |m| ~"http://github.com/treeman/rustbot");
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

    register_callbacks(irc);

    run(irc, handle);
}

