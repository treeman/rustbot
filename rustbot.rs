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

struct Irc {
    sock: @socket::TcpSocketBuf,
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
    return move recv.trim();
}

fn connect(server: ~str, port: uint, nickname: ~str, username: ~str, realname: ~str) -> ~Irc {
    let resolution = match ip::get_addr(server, iotask::spawn_iotask(task::task())) {
        Ok(m) => copy m,
        Err(_) => {
            fail ~"Host matching failed";
        }
    };
    let host = resolution.last();
    let task = iotask::spawn_iotask(task::task());

    let res = socket::connect(move host, port, task);

    let unbuffered = result::unwrap(move res);
    let sock = @socket::socket_buf(move unbuffered);

    send_raw(sock, ~"NICK " + nickname);
    send_raw(sock, ~"USER " + username + " 0 * :" + realname);

    ~Irc { sock: sock }
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

    let irc = connect(copy server, port, copy nickname, copy username, copy realname);

    loop {
        let recv = read_line(irc.sock);

        if recv.starts_with("PING") {
            send_raw(irc.sock, ~"PONG :" + recv.slice(6, recv.len()));
        }
    };
}

