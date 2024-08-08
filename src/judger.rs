mod comm;
use comm::Server;
use inquire::{MultiSelect, Text};
use core::{fmt, panic};
use std::net::TcpStream;
mod villager;
use villager::VillagerRole;
mod werewolf;
use werewolf::WerewolfRole;
mod hunter;
use hunter::HunterRole;
use rand::{seq::SliceRandom, thread_rng};

pub struct Judger {
    characters: Vec<Box<dyn Role>>,
    players: Vec<Player>,
    player_num: usize,
    idents: Vec<(identity, usize)>,
    bind_addr: String,
}

trait Role {
    fn born(&self, players: Vec<&mut Player>) {}
    fn day(&self, players: Vec<&mut Player>) {}
    fn night(&self, players: Vec<&mut Player>) {}
    fn die(&self, players: Vec<&mut Player>) {}
}

enum LifeStatus {
    Alive,
    Dead,
    NewDead,
}

struct Player {
    ser: TcpStream,
    id: usize,
    status: LifeStatus,
    place: identity,
    username: String,
}


/// 玩家的身份。
#[derive(Clone)]
enum identity {
    Villager,
    Werewolf,
    Hunter,
    Raw,
}

use identity::{Villager, Werewolf, Hunter, Raw};

impl fmt::Display for identity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Villager => write!(f, "村民"),
            Werewolf => write!(f, "狼人"),
            Hunter => write!(f, "猎人"),
            Raw => write!(f, "未分配"),
        }
    }
}

impl Judger {
    pub fn new() -> Self {
        Judger {
            characters: Vec::new(),
            players: Vec::new(),
            player_num: 0,
            idents: Vec::new(),
            bind_addr: String::new(),
        }
    }

    fn map_role(x: &identity) -> Box<dyn Role> {
        match x {
            Villager => Box::new(VillagerRole::new()),
            Werewolf => Box::new(WerewolfRole::new()),
            Hunter => Box::new(HunterRole::new()),
            Raw => panic!("roles error"),
        }
    }

    fn get_option(&mut self) {
        self.bind_addr = Text::new("绑定地址").prompt().unwrap();

        let opt_list = vec!["猎人"];
        let role_list = vec![Hunter,];
        let opt = MultiSelect::new("配置", opt_list.clone()).prompt().unwrap();

        let mut enabled_roles: Vec<_> = opt_list
            .iter()
            .enumerate()
            .filter_map(|(i, s)| {
                opt
                .iter()
                .find(|x| *x == s)
                .map(|_| role_list[i].clone())
            })
            .collect();
        enabled_roles = vec![Villager, Werewolf].into_iter()
            .chain(enabled_roles.into_iter()).collect();

        for s in enabled_roles.iter() {
            let num = Text::new(&format!("{} 的数量", *s)).prompt().
                unwrap().parse::<usize>().unwrap();
            self.player_num += 1;
            self.idents.push((s.clone(), num));
        }

        self.characters = enabled_roles.iter()
            .map(Self::map_role)
            .collect();
    }

    fn build_connect(&mut self) {
        let ser = Server::new(&self.bind_addr, self.player_num);
        let clients = ser.get_stream();
        self.players = clients
            .into_iter()
            .enumerate()
            .map(|(i, s)| Player{
                ser: s,
                id: i,
                status: LifeStatus::Alive,
                place: Raw,
                username: String::new(),
            })
            .map(move |mut x| {
                Server::send(&mut x.ser, "?");
                x.username = Server::rec(&mut x.ser);
                x
            })
            .collect();
    }

    fn assign_role(&mut self) {
        self.players.shuffle(&mut thread_rng());
        let mut cnt = 0;
        for (iden, num) in &self.idents {
            let cha = Self::map_role(iden);
            cha.born(self.players.iter_mut().skip(cnt).take(*num).collect());
            cnt += num;
        }
    }

    /// 处理夜晚的事件。夜晚事件要按照 characters 中的顺序理。
    fn night(characters: &mut Vec<Box<dyn Role>>, players: &mut Vec<Player>) {}

    fn deal_death(characters: &mut Vec<Box<dyn Role>>, players: &mut Vec<Player>) {}

    fn day(characters: &mut Vec<Box<dyn Role>>, players: &mut Vec<Player>) {}

    fn run(&mut self) {
        loop {
            Self::night(&mut self.characters, &mut self.players);
            Self::deal_death(&mut self.characters, &mut self.players);
            Self::day(&mut self.characters, &mut self.players);
            Self::deal_death(&mut self.characters, &mut self.players);
        }
    }

    pub fn init(&mut self) {
        self.get_option();
        self.build_connect();
        self.assign_role();
        self.run();
    }
}
