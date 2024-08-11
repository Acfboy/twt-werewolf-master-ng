use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use super::Responder;
use super::Identity;
use super::LifeStatus;

const NET_ERR: &str = "network error";

pub struct Human {
    username: String,
    role: Identity,
    client: TcpStream,
    id: usize,
    status: LifeStatus,
}

fn reley_prev(clients: &mut Vec<Human>) {
    clients.iter_mut().for_each(|x| x.send("+"));
}

fn return_num(client: &mut Human, pos: usize, len: usize) {
    client.send_number(pos);
    client.send_number(len);
}

pub fn build_connect(addr: &String, num: usize) -> Vec<Box<dyn Responder>> {
    let listener = TcpListener::bind(&addr).expect(NET_ERR);
    let mut clients = Vec::new();
    let mut cnt = 0;
    for c in listener.incoming() {
        if let Ok(stream) = c {
            cnt += 1;
            println!("{} connected", stream.peer_addr().unwrap());
            let mut cur = Human::new(stream);
            reley_prev(&mut clients);
            return_num(&mut cur, cnt, num);
            clients.push(cur);
            if cnt == num { break; }
        }
    }
    clients.into_iter().map(|x| Box::new(x) as Box<(dyn Responder)>).collect()
}

impl Human {
    pub fn new(client: TcpStream) -> Self {
        Human { 
            client, 
            role: Identity::Raw,
            username: String::new(),
            id: 0,
            status: LifeStatus::Alive,
        }
    }

    fn read_line(&mut self) -> Vec<u8> {
        let mut buf: [u8; 1] = [0];
        let mut res = Vec::new();
        loop {
            match self.client.read_exact(&mut buf) {
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
}

impl Responder for Human {
    fn rec(&mut self) -> String {
        let msg = self.read_line();
        let res = String::from_utf8(msg).unwrap();
        #[cfg(debug_assertions)]
        println!("rec {}", res);
        res
    }

    fn send(&mut self, msg: &str) {
        let msg = format!("{}\x07",msg);
        #[cfg(debug_assertions)]
        println!("sended {}", msg);
        self.client.write_all(msg.as_bytes()).unwrap();
    }

    fn set_status(&mut self, s: LifeStatus) {
        self.status = s;
    }

    fn status(&self) -> LifeStatus {
        self.status
    }

    fn set_role(&mut self, r: Identity) {
        self.role = r;
    }

    fn role(&self) -> Identity {
        self.role.clone()
    }

    fn set_name(&mut self) {
        self.username = self.rec_text();
    }

    fn name(&self) -> String {
        format!("{}（{} 号）", self.username, self.id)
    }

    fn set_id(&mut self, id: usize) {
        self.id = id;
    }

    fn get_id(&self) -> usize {
        self.id
    }
}