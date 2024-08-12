//! 法官。获得服务器端各种设置并主持游戏。  
//! 
//! 法官与玩家通信应遵循以下原则：
//!     - 在一次与玩家的互动结束后调用 [`Responder::send_end`] 发送结束信号。
//!     - 在玩家端不能直接确定互动是否应当开始的情况下开始互动调用 [`Responder::send_begin`] 发送开始信号。其它时候不发送。
//! 
//! 已经死亡的玩家不会理会开始和结束信号，只会显示消息。
//! 法官在发送角色发言、投票结果时也应发送给已经死亡的玩家。发送开始和结束信号时不需要考虑已经死亡的玩家。

use std::iter;

use inquire::{MultiSelect, Text};
use rand::{seq::SliceRandom, thread_rng};

pub mod roles;
use roles::LifeStatus;
use roles::DeathReason;
use roles::RoleGroup;
use roles::Responder;
use roles::responder;
use roles::Identity::{*, self};
#[allow(unused_imports)]
use roles::responder::debug_bot::DebugBot;
#[allow(unused_imports)]
use roles::responder::doubao_ai::Doubao;
pub mod log;
use log::Log;

/// 持有 [`Responder`] 特型对象的所有权。
type RespBoxes = Vec<Box<dyn Responder>>;
/// 由指向 [`Responder`] 的引用构成的向量。
type RespBoxesMut<'a> = Vec<&'a mut Box<dyn Responder>>;

/// 法官类型。
/// - `players` 存储接入的 [`Responder`] 的特型对象。
/// - `groups` 存储 [`RoleGroup`] 特型对象，用于实现角色的行为。
/// - `enabled_roles` 以 (身份, 数量) 的格式表示每个启用角色及其数量。
pub struct Judger {
    players: RespBoxes,
    groups: Vec<Box<dyn RoleGroup>>,
    bind_addr: String,
    enabled_roles: Vec<(Identity, usize)>,
    player_num: usize,
    ai_num: usize,
    log: Log,
}

impl Judger {
    pub fn new() -> Self {
        Judger {
            players: Vec::new(),
            groups: Vec::new(),
            bind_addr: String::new(),
            enabled_roles: Vec::new(),
            player_num: 0,
            ai_num: 0,
            log: Log::new(),
        }
    }

    fn get_bind_addr(&mut self) {
        #[cfg(not(debug_assertions))]
        {
            self.bind_addr = Text::new("绑定地址").prompt().unwrap();
        }
        #[cfg(debug_assertions)]
        {
            self.bind_addr = "127.0.0.1:8080".to_string();
        }
    }

    /// 获得启用的角色。狼人和平民是狼人杀的核心，不在选项内，直接加入。
    /// 这个函数中只获取启用的角色。角色数量和接入 AI 数量到 [`get_nums`] 中设置。
    fn get_enabled(&mut self) {
        let opt_list = vec!["猎人"];
        let role_list = vec![Hunter,];
        let opt = MultiSelect::new("配置", opt_list.clone()).prompt().unwrap();
        self.enabled_roles = opt_list
            .iter()
            .enumerate()
            .filter_map(|(i, s)| {
                opt
                .iter()
                .find(|x| *x == s)
                .map(|_| role_list[i].clone())
            })
            .chain(vec![Villager, Werewolf].into_iter())
            .zip(iter::repeat(0usize))
            .collect();
    }

    /// 获取角色和 AI 的数量。
    fn get_nums(&mut self) {
        for (ident, x) in self.enabled_roles.iter_mut() {
            let num = Text::new(&format!("{} 的数量", ident)).prompt().
                unwrap().parse::<usize>().unwrap();
            self.player_num += num;
            *x = num;
        }
        self.ai_num = Text::new("接入 AI 数量").prompt().unwrap().parse().unwrap();
        assert!(self.ai_num <= self.player_num);
    }

    /// 设置绑定地址和接入角色。
    fn get_config(&mut self) {
        self.get_bind_addr();
        self.get_enabled();
    }

    /// 获取所有设置。
    fn get_option(&mut self) {
        self.get_config();
        self.get_nums();
    }

    /// 先接入人类用户，再接入指定数目的 AI。连接完成后获得每个玩家的用户名和编号。
    fn build_connect(&mut self) {
        self.players = responder::human::build_connect(&self.bind_addr, self.player_num - self.ai_num);
        for _ in 0..self.ai_num {
            #[cfg(debug_assertions)]
            self.players.push(Box::new(DebugBot::new()));
            #[cfg(not(debug_assertions))]
            self.players.push(Box::new(Doubao::new()));
        }
        self.players.iter_mut().for_each(|x| x.set_name());
        self.players.iter_mut().enumerate().for_each(|(i, x)| x.set_id(i + 1));
    }

    /// 随机分配角色。
    fn assign_role(&mut self) {
        self.players.shuffle(&mut thread_rng());
        let mut cnt = 0;
        for (iden, num) in &self.enabled_roles {
            self.players.iter_mut().skip(cnt).take(*num)
                .for_each(|x| {
                    x.set_role(iden.clone());
                    x.send_msg(&format!("{}", iden));
                });
            cnt += num;
        }
        self.groups = self.enabled_roles
            .iter()
            .map(|x| roles::role_map(x.0.clone()))
            .collect();
        self.players.sort_by(|a, b| a.get_id().cmp(&b.get_id()));
        self.players.iter_mut().for_each(|x| {
            let msg = format!("{} {}", x.name(), x.role());
            self.log.write(&msg);
            println!("{msg:}");
        })
    }

    /// 对每个 [`RoleGroup`] 调用 [`night`](RoleGroup::night) 方法。给每个玩家发送结束信号。
    fn night(players: &mut RespBoxes, groups: &Vec<Box<dyn RoleGroup>>, log: &mut Log) {
        #[cfg(debug_assertions)]
        {
        println!("--NIGHT--");
        }
        log.write("--NIGHT--");
        groups.iter().for_each(|x| x.night(
            players.iter_mut().collect(),
            log
        ));
        players.iter_mut().for_each(|x| x.send_end());
    }

    /// 主持白天的审判法庭。传入所有用户。
    fn court(players: &mut RespBoxes, log: &mut Log) {
        let mut voters: RespBoxesMut = players.iter_mut()
            .filter(|x| x.status() == LifeStatus::Alive)
            .collect();
        let list: Vec<_> = voters.iter()
            .map(|x| (x.get_id(), x.name()))
            .collect();
        let tar = responder::vote(&mut voters, list, "选择处决对象".to_string(), log);
        for c in players.iter_mut() {
            if c.get_id() == tar {
                c.set_status(LifeStatus::NewDeath(DeathReason::Normal));
                break;
            }
        }
    }

    /// 白天。由讨论和投票两部组成。白天的开始是明确的，不需要发送开始信号。
    fn day(players: &mut RespBoxes, log: &mut Log) {
        #[cfg(debug_assertions)]
        {
        println!("--Day--");
        }
        log.write("--Day--");
        responder::chat(players.iter_mut().collect(), log);
        Self::court(players, log);
    }

    /// 将玩家分成平民、狼人和神职三组，返回指向它们的可变引用构成的向量。
    fn devide(players: &mut RespBoxes) -> (RespBoxesMut, RespBoxesMut, RespBoxesMut) {
        let living: Vec<_> = players.iter_mut()
            .filter(|x| x.status() == LifeStatus::Alive)
            .collect();
        let (wolves, others): (Vec<_>, Vec<_>) = living
            .into_iter().partition(|x| x.role() == Werewolf);
        let (men, clergies): (Vec<_>, Vec<_>) = others.into_iter()
            .partition(|x| x.role() == Villager);
        (wolves, men, clergies)
    }

    /// 判断游戏是否结束。
    fn is_over(&mut self) -> bool {
        let (wolves, men, clergies) = Self::devide(&mut self.players);
        let mut msg = String::new();
        if wolves.len() >= men.len() + clergies.len() || 
           self.enabled_roles.len() > 2 && clergies.is_empty() {
            msg = "狼人胜利".to_string();
        }
        else if wolves.is_empty() {
            msg = "好人胜利".to_string();
        }
        self.players.iter_mut()
            .for_each(|x| {
                if msg.is_empty() {
                    x.coutinue_game();
                }
                else {
                    x.game_over(msg.clone());
                }
            });
        self.log.write(&msg);
        !msg.is_empty()
        
    }

    /// 计算使用豆包大模型产生的成本。
    /// 豆包轻量级 4k 上下文的收费是提示词 0.0003 元 / 千 tokens，输出 0.0006 元 / 千 tokens，使用其它大模型应相应修改。  
    /// 不使用大模型花费均为 0。
    fn calc_cost(players: &mut RespBoxes, log: &mut Log) {
        let mut inp = 0;
        let mut out = 0;
        players.iter_mut().for_each(|x| {
            let cost = x.cost();
            inp += cost.0;
            out += cost.1;
        });
        let cost = (inp as f32) / 1000.0 * 0.0003 + (out as f32) / 1000.0 * 0.0006;
        let msg = format!("inp {} out {} total {} cost {}", inp, out, inp + out, cost);
        println!("{msg:}");
        log.write(&msg);
    }

    /// 循环处理白天黑夜，结束时计算开销。
    fn run(&mut self) {
        loop {
            Self::night(&mut self.players, &self.groups, &mut self.log);
            roles::check_death(&mut self.players, &mut self.log);
            if self.is_over() { break; }
            Self::day(&mut self.players, &mut self.log);
            roles::check_death(&mut self.players, &mut self.log);
            if self.is_over() { break; }
        }
        Self::calc_cost(&mut self.players, &mut self.log);
    }

    /// 服务端程序入口。
    pub fn init(&mut self) {
        self.get_option();
        self.build_connect();
        self.assign_role();
        self.run();
    }
}
