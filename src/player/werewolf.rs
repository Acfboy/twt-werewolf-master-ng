use super::{comm::{vote, Client, Message}, pause};

pub struct Werewolf { }

impl super::Role for Werewolf {
    fn born(&self, id: u64, username: String)  {
        println!("{}（{} 号）你的角色是 狼人。", username, id);
    }
    fn night(&self, cli: &mut Client) {
        cli.rec();
        loop {
            let msg = cli.receive();
            match msg {
                Message::Begin => vote(cli),
                Message::Text(s) => println!("{s:}"),
                Message::End => break,
                _ => panic!("night"),
            }
        }
        let _night_over = cli.rec();
        pause();
    }
}

impl Werewolf {
    pub fn new() -> Self { Werewolf {} }
}