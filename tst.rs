#![feature(globs)]

use std::*;
use std::io::*;

fn write_line(stream: &mut LineBufferedWriter<TcpStream>, s: &str) {
    println!("> {}", s);
    match stream.write_line(s) {
        Err(e) => {
            println!("Error: {}", e);
        }
        _ => (),
    }
}

fn read_line(stream: &mut BufferedReader<TcpStream>) -> Option<String> {
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

fn identify(stream: &mut LineBufferedWriter<TcpStream>) {
    write_line(stream, "NICK rustbot");
    write_line(stream, "USER rustbot 8 * :rustbot");
}

fn main() {
    //let mut args = os::args();
    //let binary = args.shift();

    let (tx, rx) = channel();

    let mut tcp = TcpStream::connect("irc.quakenet.org", 6667).unwrap();

    // Read input from stdin
    let tx2 = tx.clone(); // FIXME
    let tcp2 = tcp.clone(); // FIXME
    spawn(proc() {
        let mut tcp = tcp2.clone();
        for line in io::stdin().lines() {
            // FIXME prettier...
            let s : String = line.unwrap();
            let x = s.as_slice().trim();
            println!("stdin: {}", x);

            if x == "quit" {
                // FIXME quit routine
                tx2.send(format!("QUIT :Gone for repairs")); // FIXME printing routine
                tcp.close_read();
                tcp.close_write();
                drop(tcp.clone());
                break;
            }
        }
        println!("Quitting stdin reader");
    });

    // Read input from irc
    let reader = BufferedReader::new(tcp.clone());
    let tx3 = tx.clone(); // FIXME
    spawn(proc() {
        let mut reader = reader; // bug workaround
        loop {
            match read_line(&mut reader) {
                Some(x) => {
                    // FIXME text handling somewhere else
                    let s = x.as_slice().trim();
                    if s.starts_with("PING") {
                        let res = s.slice(6, s.len());
                        tx3.send(format!("PONG :{}", res));
                    }
                }
                None => break
            }
        }
        println!("Quitting irc reader");
    });

    // Write to irc
    let mut stream = LineBufferedWriter::new(tcp.clone());
    identify(&mut stream);
    spawn(proc() {
        let mut stream = stream; // bug workaround
        for x in rx.iter() {
            write_line(&mut stream, x.as_slice());
        }
        println!("Exiting irc writer");
    });
}

