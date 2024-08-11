use inquire::Text;
mod comm;
use comm::Client;
use indicatif::{ProgressBar, ProgressStyle};
use crossterm::terminal::{Clear, ClearType};
use std::io::{self, stdout, Read};
mod villager;
use villager::Villager;
mod werewolf;
use werewolf::Werewolf;
mod hunter;
use hunter::Hunter;
use comm::Message;

pub struct Player {
    cli: Client,
    id: u64,
    username: String,
    role: Box<dyn Role>,
}

trait Role {
    fn born(&self, id: u64, username: String) {}

    fn night(&self, cli: &mut Client)  {
        let _night_over = cli.rec();
    }

    fn day(&self, cli: &mut Client) {
        cli.rec();
        loop {
            let msg = cli.receive();
            match msg {
                Message::Begin => comm::vote(cli),
                Message::Text(s) => println!("{s:}"),
                Message::End => break,
                _ => panic!("error rece type"),
            }
        }
    }

    fn dead(&self, cli: &mut Client) {
        let msg = cli.receive().unwrap();
        let inq = Text::new(&msg).prompt().unwrap();
        cli.transimit(Message::Text(inq));
        cli.rec();
    }
}


/// 没有分配角色前的角色，实现了 Role trait。
struct RawRole { }

impl Role for RawRole { }

fn pause() {
    let mut stdin = io::stdin();
    println!("按回车继续。");
    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}

impl Player {
    pub fn new() -> Self {
        let addr = Text::new("服务器地址").prompt().unwrap();
        let username = Text::new("昵称").prompt().unwrap();
        let mut cli = Client::new(addr);
        let id = cli.rec_number();
        let role = Box::new(RawRole {});
        Player { cli, id, username, role }
    }

    fn wait_connect(&mut self) {
        let mut cur_player_cnt = self.id;
        let player_num = self.cli.rec_number();
        println!("等待其它玩家连接");
        let style = ProgressStyle::with_template("{bar:20} {pos:}/{len:}").unwrap();
        let pb = ProgressBar::new(player_num).with_style(style);
        pb.set_position(cur_player_cnt);
        while cur_player_cnt != player_num {
            self.cli.rec();
            pb.inc(1);
            cur_player_cnt += 1;
        }
        pb.finish();
    }

    fn reply_username(&mut self) {
        self.cli.transimit(Message::Text(self.username.clone()));
    }

    fn map_role(s: String) -> Box<dyn Role> {
        match s.as_str() {
            "平民" => Box::new(Villager::new()),
            "狼人" => Box::new(Werewolf::new()),
            "猎人" => Box::new(Hunter::new()),
            _ => panic!("error communicate"),
        }
    }

    fn get_role(&mut self) {
        let role_name = self.cli.receive().unwrap();
        self.role = Self::map_role(role_name);
        self.role.born(self.id, self.username.clone());
    }

    pub fn init(mut self) {
        self.wait_connect();
        self.reply_username();
        self.get_role();
        self.play();
    }

    fn watch(&mut self, s: String) {
        todo!();
    }

    /// 检查事件执行后的状态。
    fn is_over(cli: &mut Client) {
        let status = cli.receive();
        match status {
            Message::End => return,
            _ => {
                println!("{}", status.unwrap());
                pause();
                std::process::exit(0);
            }
        }
    }

    fn check_death(role: &mut Box<dyn Role>, cli: &mut Client) {
        loop {
            let msg = cli.receive();
            match msg {
                Message::Begin => role.dead(cli),
                Message::Text(s) => println!("{s:}"),
                Message::End => break,
                _ => panic!("error rece type"),
            }
        }
    }

    fn clear() {
        crossterm::execute!(stdout(), Clear(ClearType::All)).unwrap();
    }

    fn play(&mut self) {
        pause();
        Self::clear();
        println!("天黑请闭眼。");
        loop {
            self.role.night(&mut self.cli);
            Self::clear();
            println!("天亮了。");
            Self::check_death(&mut self.role, &mut self.cli);
            Self::is_over(&mut self.cli);
            self.role.day(&mut self.cli);
            Self::check_death(&mut self.role, &mut self.cli);
            Self::is_over(&mut self.cli);
            Self::clear();
            println!("天黑请闭眼。");
        }
    }
}
