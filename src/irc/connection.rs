use std::io::*;

// Events will be handled by main irc handler.
// Quit is needed as a special case to close down the program.
pub enum ConnectionEvent {
    Output(String),
    Received(String),
    Quit,
}

// A connection to a server.
pub struct ServerConnection {
    pub tcp: TcpStream,
    pub host: String,
    pub port: u16,
    pub tx: Sender<ConnectionEvent>,
    pub rx: Receiver<ConnectionEvent>,
}

impl ServerConnection {
    // Will simply fail if we cannot connect.
    // FIXME in the future, return error code.
    // But we need to use multiple servers for that to be useful.
    pub fn new(host: &str, port: u16) -> ServerConnection {
        let addr = format!("{}:{}", host, port);
        let tcp = match TcpStream::connect(addr[]) {
            Ok(x) => x,
            Err(e) => { panic!("{}", e); },
        };
        println!("Connected to {}:{}", host, port);

        let (tx, rx) = channel();
        ServerConnection {
            tcp: tcp,
            host: host.to_string(),
            port: port,
            tx: tx,
            rx: rx,
        }
    }

    // Close tcp connection.
    // Will cause all readers and writers to exit, possibly with safe errors.
    pub fn close(mut self) {
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
            Some(x)
        },
        Err(x) => {
            println!("error reading from stream: {}", x);
            None
        }
    }
}

