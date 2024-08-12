use crate::judger::roles::{Identity, LifeStatus};
use rand::Rng;

use super::Responder;

pub struct DebugBot {
    id: usize,
    status: LifeStatus,
    role: Identity,
    username: String,
}

impl DebugBot {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            id: 0,
            status: LifeStatus::Alive,
            role: Identity::Raw,
            username: String::new(),
        }
    }
}

impl Responder for DebugBot {
    fn send(&mut self, _msg: &str) {}

    fn rec(&mut self) -> String { 
        panic!("this is ai");
    }

    fn send_number(&mut self, _x: usize) {}

    fn rec_number(&mut self) -> usize {
        panic!("this is ai");
    }

    fn send_begin(&mut self) {}

    fn send_end(&mut self) {}

    fn send_msg(&mut self, msg: &str) {
        self.send(&format!("m{}", msg));
    }

    
    fn rec_text(&mut self) -> String {
        String::from("好人，过。")
    }
    
    fn send_json(&mut self, _jstr: &str) { 
        panic!("this is ai")
    }

    fn vote(&mut self, _msg: &str, list: Vec<(usize, String)>) -> (String, usize) {
        let mut rng = rand::thread_rng();
        let tar: usize = rng.gen::<usize>() % list.len();
        (format!("{} -> {}", self.name(), list[tar].1), list[tar].0)
    }

    fn role(&self) -> Identity {
        self.role.clone()
    }

    fn set_role(&mut self, r: Identity) {
        self.role = r;
    }

    fn status(&self) -> LifeStatus {
        self.status
    }

    fn set_status(&mut self, s: LifeStatus) {
        self.status = s;
    }

    fn set_name(&mut self) {
        self.username = "舔一舔".to_string();
    }

    fn name(&self) -> String { 
        format!("{}（{} 号）", &self.username, self.id)
    }

    fn set_id(&mut self, id: usize) {
        self.id = id;
    }

    fn get_id(&self) -> usize { 
        self.id
    }

    fn coutinue_game(&mut self) {}

}

