use std::map;
use std::map::*;
use std::sort;
use std::sort::*;

struct Irc {
    sock: @socket::TcpSocketBuf,
    mut msg_cb: ~[@fn(m: &IrcMsg)],
    mut privmsg_cb: ~[@fn(m: &PrivMsg)],
    cmd_cb: @HashMap<~str, @fn(m: &CmdMsg) -> ~str>,
    bare_cmd_cb: @HashMap<~str, @fn() -> ~str>,
}

struct IrcMsg {
    irc: @Irc,
    prefix: ~str,
    code: ~str,
    param: ~str,
}

struct PrivMsg {
    irc: @Irc,
    channel: ~str,
    msg: ~str,
}

struct CmdMsg {
    irc: @Irc,
    channel: ~str,
    cmd: ~str,
    args: ~[~str],
    arg: ~str,
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

// Split a string into components and build a IrcMsg object
pure fn build_irc_msg(irc: @Irc, s: &str) -> ~IrcMsg {
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

    ~IrcMsg { irc: irc, prefix: move prefix, code: move code, param: move param }
}

// Parse a PRIVMSG parameters and build a PrivMsg object
pure fn build_privmsg(irc: @Irc, s: &str) -> ~PrivMsg {
    let space = match find_str(s, " ") {
        Some(i) => i,
        None => 0,
    };

    // '#channel :msg'
    let channel = s.substr(0, space);
    // Skip :
    let msg = s.slice(space + 2, s.len());

    ~PrivMsg { irc: irc, channel: move channel, msg: move msg }
}

pure fn is_cmd(m: &PrivMsg) -> bool {
    m.msg.starts_with(".")
}

// Parse a PRIVMSG which is seen as a command and build a CmdMsg object
pure fn build_cmd(irc: @Irc, m: &PrivMsg) -> ~CmdMsg {
    let split = split_char(m.msg.slice(1, m.msg.len()), ' ');
    let cmd: ~str = split.head(); // TODO warn
    let args = split.tail(); // TODO warn
    let rest = foldl(~"", args, |a,e| a + *e + " ").trim();

    // TODO copying vectors = bad?
    ~CmdMsg {
        irc: irc,
        channel: copy m.channel,
        cmd: copy cmd,
        args: copy args,
        arg: copy rest,
    }
}

fn handle_cmd(cmd: &CmdMsg) {
    let mut irc = cmd.irc;

    // TODO warnings again
    // Find bare bone cmds
    if irc.bare_cmd_cb.contains_key(cmd.cmd) {
        // Load and execute
        let a = irc.bare_cmd_cb.get(cmd.cmd)().trim();

        if a != ~"" {
            // Split up at \n for multi line responses
            let msgs = lines(a);
            for msgs.each |m| {
                privmsg(irc, cmd.channel, *m);
            }
        }
    }

    // Find cmds that handle their arguments
    if irc.cmd_cb.contains_key(cmd.cmd) {
        // Load and execute
        let a = irc.cmd_cb.get(cmd.cmd)(cmd).trim();

        if a != ~"" {
            // Split up at \n for multi line responses
            let msgs = lines(a);
            for msgs.each |m| {
                privmsg(irc, cmd.channel, *m);
            }
        }
    }
}

fn cmds(m: &CmdMsg) -> ~str {
    let irc = m.irc;

    let mut v: ~[~str] = ~[];

    v.push(~"cmds");

    // Collect
    for irc.bare_cmd_cb.each_key |c| {
        v.push(c);
    }
    for irc.cmd_cb.each_key |c| {
        v.push(c);
    }

    // Sort
    quick_sort(v, |a, b| a < b);

    // Output, functional programming ftw!
    foldl(~"", v, |a, b| a + ~" ." + *b).trim()
}

fn handle_received(m: &IrcMsg) {
    // Handle callbacks
    // WTH?!
    /*for each_mut(m.irc.msg_cb) |cb| {
        let mut cb_ = *cb;
        cb_(m);
    }*/

    match m.code {
        ~"PRIVMSG" => {
            let msg = build_privmsg(m.irc, m.param);

            if is_cmd(msg) {
                let cmd = build_cmd(m.irc, msg);

                handle_cmd(cmd);
            }
        }
        _ => ()
    }
}

fn run(irc: @Irc, f: fn(irc: @Irc, m: &IrcMsg))
{
    // Register simple handlers
    register_cmd(irc, ~"cmds", cmds);

    loop {
        let recv = read_line(irc.sock);

        if recv.starts_with("PING") {
            send_raw(irc.sock, ~"PONG :" + recv.slice(6, recv.len()));
        }
        else {
            let m = build_irc_msg(irc, recv);

            f(irc, m);
            handle_received(m);
        }
    };
}

fn register_msg(irc: &Irc, f: @fn(m: &IrcMsg))
{
    irc.msg_cb.push(f);
}

fn register_privmsg(irc: &Irc, f: @fn(m: &PrivMsg))
{
    irc.privmsg_cb.push(f);
}

fn register_cmd(irc: &Irc, cmd: ~str, f: @fn(m: &CmdMsg) -> ~str)
{
    irc.cmd_cb.insert(cmd, f);
}

fn register_bare_cmd(irc: &Irc, cmd: ~str, f: @fn() -> ~str)
{
    irc.bare_cmd_cb.insert(cmd, f);
}

fn connect(server: &str, port: uint) -> @Irc
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

    @Irc {
        sock: sock,
        msg_cb: ~[],
        privmsg_cb: ~[],
        cmd_cb: @HashMap::<~str, @fn(m: &CmdMsg) -> ~str>(),
        bare_cmd_cb: @HashMap::<~str, @fn() -> ~str>(),
    }
}

