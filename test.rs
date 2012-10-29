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

fn split(s: &str) -> ~[~str] {
    ~[~"a", ~"b", ~"c"]
}

fn main() {
    let s = ":beginning 123 the rest";
    let m = split(s);

    for m.each |s| {
        println(*s);
    }
}

