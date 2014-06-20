use std::io::*;

// A connection to a server.
pub struct ServerConnection {
    pub tcp: TcpStream,
    pub host: String,
    pub port: u16,
}

impl ServerConnection {
    // Will simply fail if we cannot connect.
    // FIXME in the future, return error code.
    // But we need to use multiple servers for that to be useful.
    pub fn new(host: &str, port: u16) -> ServerConnection {
        let tcp = match TcpStream::connect(host, port) {
            Ok(x) => x,
            Err(e) => { fail!("{}", e); },
        };
        println!("Connected to {}:{}", host, port);
        ServerConnection { tcp: tcp, host: host.to_string(), port: port }
    }

    // Close tcp connection.
    // Will cause all readers and writers to exit, possibly with safe errors.
    pub fn close(&mut self) {
        match self.tcp.close_read() {
            Err(e) => println!("Error closing read: {}", e),
            _ => (),
        };
        match self.tcp.close_write() {
            Err(e) => println!("Error closing write: {}", e),
            _ => (),
        };
        drop(self.tcp.clone());
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

