//! 实现底层通讯。

use std::net::TcpStream;
use std::io::{Read, Write};
use serde_json;
use inquire::Select;

const NET_ERR: &str = "network error";

pub struct Client {
    pub stream: TcpStream,
}

/// 收到信息的类型。
pub enum Message {
    Text(String),
    Number(usize),
    Json(String),
    Begin,
    End,
}

impl Message {
    /// 方便从消息中获得原始信息和 json 字符串。
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

    /// 接收一条消息。TcpStream 是流式的，没有消息分隔。这里以蜂鸣器控制字符分隔。
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

    /// 接收原始信息。
    pub fn rec(&mut self) -> String {
        let msg = Self::read_line(&mut self.stream);
        let res = String::from_utf8(msg).unwrap();
        // #[cfg(debug_assertions)]
        // println!("{}: {}", "Received", res);
        res
    }

    /// 接收数字，收到的不是数字说明服务端出了大问题，直接 panic。
    pub fn rec_number(&mut self) -> u64 {
        let res = self.receive();
        if let Number(x) = res {
            x as u64
        }
        else { panic!("not a number"); }
    }

    /// 发送原始信息。
    pub fn send(&mut self, message: &str) {
        let message = format!("{}\x07", message);
        self.stream.write_all(message.as_bytes()).unwrap();
        // #[cfg(debug_assertions)]
        // println!("{}: {}", "Sended", message);
    }

    /// 发送 [`Message`] 封装好的信息。
    pub fn transimit(&mut self, x: Message) {
        match x {
            Text(s) => self.send(&format!("t{}", s)),
            Number(n) => self.send(&format!("n{}", n)),
            Json(s) => self.send(&format!("j{}", s)),
            _ => panic!("can't send"),
        }
    }

    /// 接收消息，用 [`Message`] 封装好。
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

    /// 接收一次开始信号。
    pub fn begin(&mut self) {
        let res = self.receive();
        match res {
            Begin => return,
            _ => panic!("begin error"),
        }
    }

    /// 接收一次结束信号
    pub fn end(&mut self) {
        let res = self.receive();
        match res {
            End => return,
            _ => panic!("end error"),
        }
    }
}


/// 回应单次投票。
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
