use inquire::Text;

use super::comm::{vote, Client, Message};

pub struct Hunter { }

impl super::Role for Hunter {
    fn born(&self, id: u64, username: String)  {
        println!("{}（{} 号）你的角色是 猎人", username, id);
    }

    fn dead(&self, cli: &mut Client) {
        let inq = Text::new(&cli.receive().unwrap()).prompt().unwrap();
        cli.transimit(Message::Text(inq));
        vote(cli);
        let _detail = cli.rec();
        cli.rec();
    }
}

impl Hunter {
    pub fn new() -> Self { Hunter {} }
}