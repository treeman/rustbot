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

struct IrcMsg {
    prefix: ~str,
    code: ~str,
    param: ~str,
}

struct PrivMsg {
    channel: ~str,
    msg: ~str,
}

// Split a string into components
pure fn parse_irc_msg(s: &str) -> IrcMsg {
    let mut space = 0;
    let mut last = 0;

    // If first is ':', find next space
    if s.starts_with(":") {
        space = match find_str(s, " ") {
            Some(i) => i,
            None => 0,
        };
    }

    // Create prefix
    let mut prefix = ~"";
    if space != 0 {
        prefix = s.slice(1, space);
        last = space + 1;
    }

    // Find space between cmd and parameters
    space = match find_str_from(s, " ", last) {
        Some(i) => i,
        None => 0,
    };

    let code = s.slice(last, space);
    let param = s.slice(space + 1, s.len());

    IrcMsg { prefix: move prefix, code: move code, param: move param }
}

pure fn parse_privmsg(s: &str) -> PrivMsg {
    let space = match find_str(s, " ") {
        Some(i) => i,
        None => 0,
    };

    // '#channel :msg'
    let channel = s.substr(0, space);
    // Skip :
    let msg = s.slice(space + 2, s.len());

    PrivMsg { channel: move channel, msg: move msg }
}

fn main() {
    let tst = ~[
        ~":port80b.se.quakenet.org 003 rustbot :This server was created Mon Mar 24 2008 at 23:41:47 CET",
        ~"PRIVMSG #madeoftree :hello rustbot!",
    ];

    for tst.each |s| {
        let m = parse_irc_msg(*s);
        println(fmt!("'%s' '%s' '%s'", m.prefix, m.code, m.param));

        if m.code == ~"PRIVMSG" {
            let what = parse_privmsg(m.param);

            println(fmt!("msg -> '%s' '%s'", what.channel, what.msg));
        }
    }

    let s = "asdf asd as a";

    let split = split_char(s.slice(0, s.len()), ' ');
    for split.each |s| {
        println(*s);
    }
}

