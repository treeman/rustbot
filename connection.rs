#![feature(globs)]

use std::*;
use std::io::*;

// A connection to a server.
struct ServerConnection {
    tcp: TcpStream,
    server: String,
    port: u16,
}

impl ServerConnection {
    // Will simply fail if we cannot connect.
    // FIXME in the future, return error code.
    // But we need to use multiple servers for that to be useful.
    pub fn new(server: &str, port: u16) -> ServerConnection {
        let mut tcp = match TcpStream::connect(server, port) {
            Ok(x) => x,
            Err(e) => { fail!("{}", e); },
        };
        println!("Connected to {}:{}", server, port);
        ServerConnection { tcp: tcp, server: server.to_string(), port: port }
    }
}


// Primitive write from tcp buffer.
pub fn write_line(stream: &mut LineBufferedWriter<TcpStream>, s: &str) {
    println!("> {}", s);
    match stream.write_line(s) {
        Err(e) => {
            println!("Error: {}", e);
        }
        _ => (),
    }
}

// Primitive read from tcp buffer.
pub fn read_line(stream: &mut BufferedReader<TcpStream>) -> Option<String> {
    match stream.read_line() {
        Ok(x) => {
            print!("< {}", x);
            Some(x)
        },
        Err(x) => {
            println!("error reading from stream: {}", x);
            None
        }
    }
}

// Commands for writer
pub enum WriterCommand {
    Write(String),
    Quit
}

pub struct TxWriter {
    tx: Sender<WriterCommand>,
}

impl TxWriter {
    pub fn new(tx: &Sender<WriterCommand>) -> TxWriter {
        TxWriter{ tx: tx.clone() }
    }

    pub fn write(&self, s: String) {
        self.tx.send(Write(s));
    }

    // Use for closing down
    pub fn send_quit(&self) {
        self.tx.send(Quit);
    }
}

