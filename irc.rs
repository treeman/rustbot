
struct Irc {
    sock: @socket::TcpSocketBuf,
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

fn identify(irc: &Irc, nickname: &str, username: &str, realname: &str) {
    send_raw(irc.sock, ~"NICK " + nickname);
    send_raw(irc.sock, ~"USER " + username + " 0 * :" + realname);
}

fn privmsg(irc: &Irc, channel: &str, msg: &str) {
    send_raw(irc.sock, fmt!("PRIVMSG %s :%s", channel, msg));
}

fn join(irc: &Irc, channel: &str) {
    send_raw(irc.sock, fmt!("JOIN %s", channel));
}

// Split a string into components
pure fn parse_irc_msg(s: &str) -> ~IrcMsg {
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

    ~IrcMsg { prefix: move prefix, code: move code, param: move param }
}

// Parse a PRIVMSG parameters
pure fn parse_privmsg(s: &str) -> ~PrivMsg {
    let space = match find_str(s, " ") {
        Some(i) => i,
        None => 0,
    };

    // '#channel :msg'
    let channel = s.substr(0, space);
    // Skip :
    let msg = s.slice(space + 2, s.len());

    ~PrivMsg { channel: move channel, msg: move msg }
}

fn run(irc: &Irc, f: fn(irc: &Irc, m: &IrcMsg)) {
    loop {
        let recv = read_line(irc.sock);

        if recv.starts_with("PING") {
            send_raw(irc.sock, ~"PONG :" + recv.slice(6, recv.len()));
        }
        else {
            let m = parse_irc_msg(recv);

            f(irc, m);
        }
    };
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

