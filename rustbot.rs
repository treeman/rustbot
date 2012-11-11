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

fn connect(server: &str, port: uint) -> ~Irc
{
    let resolution = match ip::get_addr(server,
        iotask::spawn_iotask(task::task()))
    {
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

    ~Irc { sock: sock }
}

fn identify(irc: &Irc, nickname: &str, username: &str, realname: &str) {
    send_raw(irc.sock, ~"NICK " + nickname);
    send_raw(irc.sock, ~"USER " + username + " 0 * :" + realname);
}

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
        prefix = s.substr(1, space - 1);
        last = space + 1;
    }

    // Find space between cmd and parameters
    space = match find_str_from(s, " ", last) {
        Some(i) => i,
        None => 0,
    };

    let code = s.substr(last, space - last);
    let param = s.substr(space + 1, s.len() - space - 1);

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
    let msg = s.substr(space + 2, s.len() - space - 2);

    PrivMsg { channel: move channel, msg: move msg }
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

    loop {
        let recv = read_line(irc.sock);

        if recv.starts_with("PING") {
            send_raw(irc.sock, ~"PONG :" + recv.slice(6, recv.len()));
        }
        else {
            let m = parse_irc_msg(recv);

            match m.code {
                ~"004" => {
                    send_raw(irc.sock, ~"JOIN #madeoftree");
                }
                ~"PRIVMSG" => {
                    let msg = parse_privmsg(m.param);

                    if msg.msg.starts_with("hello") {
                        send_raw(irc.sock, fmt!("PRIVMSG %s :hello there mister!", msg.channel));
                    }
                }
                _ => (),
            }

        }
    };
}

