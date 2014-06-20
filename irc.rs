#![feature(globs)]

use std::*;
use std::io::*;
use connection::*;

pub struct IrcConfig<'a> {
    pub server: &'a str,
    pub port: u16,
    pub nick: &'a str,
    pub descr: &'a str,
}

struct ServerConnection {
    tcp: TcpStream,
    server: String,
    port: u16,
}

impl ServerConnection {
    pub fn new(server: &str, port: u16) -> ServerConnection {
        let mut tcp = match TcpStream::connect(server, port) {
            Ok(x) => x,
            Err(e) => { fail!("{}", e); },
        };
        println!("Connected to {}:{}", server, port);
        ServerConnection { tcp: tcp, server: server.to_string(), port: port }
    }
}

pub struct Irc {
    // tx: TxWriter,
    // tcp: TcpStream, // ??
    // server + port
    // command callbacks?
    // + register functions
    conn: ServerConnection, // Multiple servers? Maybe later, but not now.
}

impl Irc {

    pub fn connect(conf: IrcConfig) -> Irc {
        Irc { conn: ServerConnection::new(conf.server, conf.port) }
    }

    pub fn run(&mut self) {
        let (tx, rx) = channel();
    }
}

struct IrcMsg {
    prefix: String,
    code: String,
    param: String,
}

struct IrcPrivMsg {
    channel: String,
    msg: String,
}

struct IrcCmdMsg {
    channel: String,
    cmd: String,
    args: String,
}

struct Cmd {
    name: String,
    args: String,
}

