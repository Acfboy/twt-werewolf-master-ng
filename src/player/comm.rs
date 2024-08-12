use std::net::TcpStream;
use std::io::{Read, Write};
use serde_json;
use inquire::Select;

const NET_ERR: &str = "network error";

pub struct Client {
    pub stream: TcpStream,
}

pub enum Message {
    Text(String),
    Number(usize),
    Json(String),
    Begin,
    End,
}

impl Message {
    pub fn unwrap(self) -> String {
        match self {
            Text(s) | Json(s) => s,
            _ => panic!("can't unwrap"),
        }
    } 
}

use Message::*;

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
                    if buf[0] == 0x7 {
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
        // #[cfg(debug_assertions)]
        // println!("{}: {}", "Received", res);
        res
    }

    pub fn rec_number(&mut self) -> u64 {
        let res = self.receive();
        if let Number(x) = res {
            x as u64
        }
        else { panic!("not a number"); }
    }

    pub fn send(&mut self, message: &str) {
        let message = format!("{}\x07", message);
        self.stream.write_all(message.as_bytes()).unwrap();
        // #[cfg(debug_assertions)]
        // println!("{}: {}", "Sended", message);
    }

    pub fn transimit(&mut self, x: Message) {
        match x {
            Text(s) => self.send(&format!("t{}", s)),
            Number(n) => self.send(&format!("n{}", n)),
            Json(s) => self.send(&format!("j{}", s)),
            _ => panic!("can't send"),
        }
    }

    pub fn receive(&mut self) -> Message {
        let msg = self.rec();
        let mut it = msg.chars();
        let typ = it.next().unwrap();
        let s = it.collect();
        match typ {
            'y' => Begin,
            'o' => End,
            'j' => Json(s),
            'm' => Text(s),
            'n' => Number(s.parse().unwrap()),
            _ => panic!("received type error"),
        }
    }

    pub fn begin(&mut self) {
        let res = self.receive();
        match res {
            Begin => return,
            _ => panic!("begin error"),
        }
    }

    pub fn end(&mut self) {
        let res = self.receive();
        match res {
            End => return,
            _ => panic!("end error"),
        }
    }
}

pub fn vote(cli: &mut Client) {
    let pmt = cli.receive().unwrap();
    let s = cli.receive().unwrap();
    let list: Vec<String> = serde_json::from_str(&s).unwrap();
    println!();
    let inq = Select::new(&pmt, list.clone()).prompt().unwrap();
    for c in list.into_iter().enumerate() {
        if c.1 == inq { cli.transimit(Number(c.0)); break; }
    }
}
