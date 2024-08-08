use std::net::TcpStream;
use std::io::{Read, Write};
use colored::*;

const NET_ERR: &str = "network error";

pub struct Client {
    pub stream: TcpStream,
}

impl Client {
    pub fn new(addr: String) -> Client {
        let stream = TcpStream::connect(&addr).expect(NET_ERR);
        Client { stream }
    }

    fn read_line(stream: &mut TcpStream) -> Vec<u8> {
        let mut buf: [u8; 1] = [0];
        let mut res = Vec::new();
        loop {
            match stream.read_exact(&mut buf) {
                Ok(_) => {
                    if buf[0] == b'\n' {
                        break;
                    }
                    res.push(buf[0]);
                }
                Err(_) => {
                    break;
                }
            }
        }
        res
    }

    pub fn rec(&mut self) -> String {
        let msg = Self::read_line(&mut self.stream);
        let res = String::from_utf8(msg).unwrap();
        // println!("{}: {}", "Received".green(), res);
        res
    }

    pub fn send(&mut self, message: &str) {
        let msessage = format!("{}\n", message);
        self.stream.write_all(message.as_bytes()).unwrap();
        // println!("{} {}: {}", "DEBUG".yellow(), "Sended".red(), message);
    }
}
