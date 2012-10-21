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

fn usage(binary: &str) {
    io::println(fmt!("Usage: %s [options]\n", binary) +
            "
Options:

    -h --help           Show this helpful screen
");
}

fn send_raw(sock: @socket::TcpSocketBuf, txt: ~str) {
    let writer = sock as Writer;
    writer.write_str(txt + "\r\n");
    writer.flush();

    println(fmt!("> %s", txt));
}

// Read a single line from socket, block until done
fn read_line(sock: @socket::TcpSocketBuf) -> ~str {
    let reader = sock as Reader;
    let recv = reader.read_line();
    println(fmt!("< %s", recv));
    return move recv;
}

fn main() {
    let mut args = os::args();
    let binary = args.shift();

    if args.is_empty() { usage(binary); return; }

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

    /*
    io::println("Running irc bot w00");

    for args.each |s| {
        io::println(*s);
    }
    */

    let resolution = match ip::get_addr("irc.quakenet.org", iotask::spawn_iotask(task::task())) {
        Ok(m) => copy m,
        Err(_) => {
            io::println("Host matching failed");
            return;
        }
    };
    let host = resolution.last();
    // println(fmt!("ip host: %?", host));

    // let host = ip::get_addr("irc.quakenet.org", iotask::spawn_iotask(task::task()));
    // println(fmt!("host: %?", host));

    let task = iotask::spawn_iotask(task::task());
    // let ip = ip::v4::parse_addr("178.79.132.147");
    // println(fmt!("ip v4: %?", ip));

    // let res = socket::connect(move ip, 6667u, task);
    let res = socket::connect(move host, 6667u, task);

    let unbuffered = result::unwrap(move res);
    let sock = @socket::socket_buf(move unbuffered);

    send_raw(sock, ~"NICK rustbot");
    send_raw(sock, ~"USER rustbot 0 * :rustbot");

    loop {
        let recv = read_line(sock);

        if recv.starts_with(~"PING") {
            send_raw(sock, ~"PONG :" + recv.slice(6, recv.len()));
        }
    };
}

