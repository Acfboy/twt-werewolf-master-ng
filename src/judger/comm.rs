use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use colored::*;

const NET_ERR: &str = "network error";

pub struct Server {
    pub clients: Vec<TcpStream>,
}

impl Server {
    pub fn new(addr: &String, num: usize) -> Server {
        let listener = TcpListener::bind(&addr).expect(NET_ERR);
        let mut clients = Vec::new();
        
        let mut cnt = 0;
        for c in listener.incoming() {
            if let Ok(mut stream) = c {
                println!("{} connected", stream.peer_addr().unwrap());
                clients.iter_mut().for_each(|x| Self::send(x, "+"));
                clients.push(stream.try_clone().expect(NET_ERR));
                cnt += 1;
                Self::send(&mut stream, &(cnt.to_string()));
                Self::send(&mut stream, &(num.to_string()));
                if cnt == num { break; }
            }
        }

        Server { clients }
    }

    pub fn get_stream(self) -> Vec<TcpStream> {
        self.clients
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

    pub fn rec(stream: &mut TcpStream) -> String {
        let msg = Self::read_line(stream);
        let res = String::from_utf8(msg).unwrap();
        // println!("{}: {}", "Received".green(), res);
        res
    }

    pub fn send(stream: &mut TcpStream, msg: &str) {
        let msg = format!("{}\n",msg);
        stream.write_all(msg.as_bytes()).unwrap();
        // println!("{}: {}", "Sended".red(), msg);
    }

}