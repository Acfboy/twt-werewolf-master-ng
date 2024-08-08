use inquire::Text;
mod comm;
use comm::Client;
use indicatif::{ProgressBar, ProgressStyle};
use crossterm::terminal::{Clear, ClearType};
use std::io::stdout;
mod villager;
use villager::Villager;
mod werewolf;
use werewolf::Werewolf;
mod hunter;
use hunter::Hunter;

pub struct Player {
    cli: Client,
    id: u64,
    username: String,
    role: Box<dyn Role>,
}


/// Over 表示一方获胜，Dead 表示自身死亡，Continue 无事发生。
enum EventResult {
    Over(String),
    Continue(String),
    Dead,
}

use EventResult::{Continue, Dead, Over};

/// 角色特型，实现角色在出生时、白天、夜晚和死亡时的行为来定义一个角色。
/// 返回一个 EventResult 以处理角色活动后自身是否死亡或整个游戏结束。
trait Role {
    fn born(&self) {
    }

    fn night(&self, cli: &mut Client) -> EventResult {
        panic!("还没写到这儿");
        Continue("天亮了。".to_string())
    }

    fn day(&self, cli: &mut Client) -> EventResult {
        Continue("".to_string())
    }

    fn dead(&self, cli: &mut Client) -> EventResult {
        todo!();
    }
}


/// 没有分配角色前的角色，实现了 Role trait。
struct RawRole { }

impl Role for RawRole { }

impl Player {
    pub fn new() -> Self {
        let addr = Text::new("服务器地址").prompt().unwrap();
        let username = Text::new("昵称").prompt().unwrap();
        let mut cli = Client::new(addr);
        let id = cli.rec().parse::<u64>().unwrap();
        let role = Box::new(RawRole {});
        Player { cli, id, username, role }
    }

    fn wait_connect(&mut self) {
        let mut cur_player_cnt = self.id;
        let player_num = self.cli.rec().parse::<u64>().unwrap();
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
        let _ = self.cli.rec();
        self.cli.send(&self.username);
    }

    fn map_role(s: String) -> Box<dyn Role> {
        match s.as_str() {
            "村民" => Box::new(Villager::new()),
            "狼人" => Box::new(Werewolf::new()),
            "猎人" => Box::new(Hunter::new()),
            _ => panic!("error communicate"),
        }
    }

    /// 获得自身角色。注意，玩家端获取角色时法官端已经进入角色的 born 方法。
    fn get_role(&mut self) {
        let role_name = self.cli.rec();
        self.role = Self::map_role(role_name);
        self.role.born();
    }

    pub fn init(mut self) {
        self.wait_connect();
        self.reply_username();
        self.get_role();
        self.play();
    }

    fn finish(s: String) {
        println!("{}", s);
        std::process::exit(0);
    }

    fn watch(&mut self, s: String) {
        todo!();
    }

    /// 检查事件执行后的状态。
    fn check(res: EventResult, role: &Box<dyn Role>, cli: &mut Client) {
        match res {
            Over(s) => Self::finish(s),
            Dead => {
                let res = role.dead(cli);
                if let Over(s) = res {
                    Self::finish(s);
                }
            }
            Continue(s) => println!("{}", s),
        }
    }

    fn day(&mut self) -> EventResult {
        todo!()
    }

    fn play(&mut self) {
        crossterm::execute!(stdout(), Clear(ClearType::All)).unwrap();
        println!("天黑请闭眼。");
        loop {
            let res = self.role.night(&mut self.cli);
            Self::check(res, &self.role, &mut self.cli);
            let res = self.role.day(&mut self.cli);
            Self::check(res, &self.role, &mut self.cli);
        }
    }
}
