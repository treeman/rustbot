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

struct IrcMsg {
    prefix: ~str,
    cmd: ~str,
    param: ~str,
}

fn split(s: &str) -> IrcMsg {
    let mut prefix = ~"";

    /*if s.starts_with(":") {
        let pos = s.find_char(' ');
        prefix = s.slice(1, pos);
    }*/

    let cmd = ~"123";
    let param = ~"rest";

    IrcMsg { prefix: move prefix, cmd: move cmd, param: move param }
}

fn main() {
    let tst = ~[
        ~":port80b.se.quakenet.org 003 rustbot :This server was created Mon Mar 24 2008 at 23:41:47 CET",
        ~"PRIVMSG #madeoftree :hello rustbot!",
    ];

    let mut s: ~str = ~"  HaRroRR ";
    s = s.trim();
    //let pos = s.find_char('a'); // Fails?!
    println(fmt!("%u", s.len()));
    println(s.to_upper());

    s.contains("aR");
    //find_str(*s, "ro"); // Fails?!

    // Fails?!
    /*match s.find_char('a') {
        Some(i) => println(fmt!("at pos %u", i)),
        None => println("not found"),
    }*/

    for tst.each |s| {
        let m = split(*s);
        println(fmt!("'%s' '%s' '%s'", m.prefix, m.cmd, m.param));
    }
}

