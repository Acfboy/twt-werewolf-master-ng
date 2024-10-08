//! 玩家端。

use inquire::Text;
pub mod comm;
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
    fn born(&self, _id: u64, _username: String) {}

    fn night(&self, cli: &mut Client)  {
        let _night_over = cli.end();
        pause();
    }

    fn day(&self, cli: &mut Client) {
        cli.begin();
        loop {
            let msg = cli.receive();
            match msg {
                Message::Begin => {
                    let inq = Text::new("发言：").prompt().unwrap();
                    cli.transimit(Message::Text(inq));
                }
                Message::Text(s) => println!("{s:}"),
                Message::End => break,
                _ => panic!("error rece type"),
            }
        }
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
        watch(cli);
    }
}

fn watch(cli: &mut Client) {
    println!("[观战模式]");
    loop {
        let msg = cli.receive();
        if let Message::Text(s) = msg {
            println!("{s:}");
            if s == "好人胜利" || s == "狼人胜利" {
                break;
            }
        }
    }
    pause();
    std::process::exit(0);
}

/// 没有分配角色前的角色。
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
        let addr ;
        #[cfg(not(debug_assertions))] 
        {
            addr = Text::new("服务器地址").prompt().unwrap();
        }
        #[cfg(debug_assertions)]
        {
            addr = "127.0.0.1:8080".to_string();
        }
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

    /// 检查游戏是否结束。
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

    /// 检查死亡。为了处理多个死亡的情形已经方便死亡提示，当收到开始信号后再进行死亡动作，其它消息直接输出。
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

    /// 清空屏幕。
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
            pause();
            println!("天黑请闭眼。");
        }
    }
}
