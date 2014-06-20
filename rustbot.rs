#![feature(globs)]

use std::*;
use std::io::*;
use connection::*;
use irc::*;

mod connection;
mod irc;

// TODO change
fn identify(stream: &mut LineBufferedWriter<TcpStream>) {
    write_line(stream, "NICK rustbot");
    write_line(stream, "USER rustbot 8 * :rustbot");
}

// Read input from stdin.
fn spawn_stdin_reader(tx: Sender<WriterCommand>) {
    spawn(proc() {
        for line in io::stdin().lines() {
            // FIXME prettier...
            let s : String = line.unwrap();
            let x = s.as_slice().trim();
            println!("stdin: {}", x);

            if x == "quit" {
                tx.send(Quit);
                break;
            }
        }
        println!("Quitting stdin reader");
    })
}

// Read input from irc.
fn spawn_irc_reader(tx: Sender<WriterCommand>, tcp: TcpStream) {
    spawn(proc() {
        let mut reader = BufferedReader::new(tcp);
        loop {
            match read_line(&mut reader) {
                Some(x) => {
                    // FIXME text handling somewhere else
                    let s = x.as_slice().trim();
                    if s.starts_with("PING") {
                        let res = s.slice(6, s.len());
                        //reader.request_write(format!("PONG :{}", res));
                        //write_line(&mut reader, format!("PONG :{}", res));
                        tx.send(Write(format!("PONG :{}", res)));
                    }
                }
                None => break
            }
        }
        println!("Quitting irc reader");
    });
}

// Write to irc.
fn spawn_irc_writer(rx: Receiver<WriterCommand>, tcp: TcpStream) {
    spawn(proc() {
        let mut stream = LineBufferedWriter::new(tcp.clone());
        let mut tcp = tcp; // bug workaround

        identify(&mut stream);
        for x in rx.iter() {
            match x {
                Write(s) => write_line(&mut stream, s.as_slice()),
                Quit => {
                    write_line(&mut stream, "QUIT :Gone for repairs"); // FIXME printing routine
                    tcp.close_read();
                    tcp.close_write();
                    drop(tcp.clone());
                    break;
                },
            }
        }
        println!("Exiting irc writer");
    });
}

fn spawn_bot() {
    // FIXME create an Irc object or something.
    let mut tcp = TcpStream::connect("irc.quakenet.org", 6667).unwrap();
    let (tx, rx) = channel();

    spawn_stdin_reader(tx.clone());
    spawn_irc_reader(tx.clone(), tcp.clone());
    spawn_irc_writer(rx, tcp);
}

fn main() {
    //let mut args = os::args();
    //let binary = args.shift();

    //spawn_bot();

    let conf = IrcConfig {
        server: "irc.quakenet.org",
        port: 6667,
        nick: "rustbot",
        descr: "https://github.com/treeman/rustbot"
    };

    let mut irc = Irc::connect(conf);
    irc.run();
}

