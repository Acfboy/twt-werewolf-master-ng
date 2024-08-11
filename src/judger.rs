use std::iter;

use inquire::{MultiSelect, Text};
use rand::{seq::SliceRandom, thread_rng};

mod roles;
use roles::LifeStatus;
use roles::RoleGroup;
use roles::Responder;
use roles::responder;
use roles::Identity::{*, self};

type RespBoxes = Vec<Box<dyn Responder>>;
type RespBoxesMut<'a> = Vec<&'a mut Box<dyn Responder>>;

#[derive(Default)]
pub struct Judger {
    players: RespBoxes,
    groups: Vec<Box<dyn RoleGroup>>,
    bind_addr: String,
    enabled_roles: Vec<(Identity, usize)>,
    player_num: usize,
}

impl Judger {
    fn get_bind_addr(&mut self) {
        self.bind_addr = Text::new("绑定地址").prompt().unwrap();
    }

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

    fn get_nums(&mut self) {
        for (ident, x) in self.enabled_roles.iter_mut() {
            let num = Text::new(&format!("{} 的数量", ident)).prompt().
                unwrap().parse::<usize>().unwrap();
            self.player_num += num;
            *x = num;
        }
    }

    fn get_config(&mut self) {
        self.get_bind_addr();
        self.get_enabled();
    }

    fn get_option(&mut self) {
        self.get_config();
        self.get_nums();
    }

    fn build_connect(&mut self) {
        self.players = responder::human::build_connect(&self.bind_addr, self.player_num);
        self.players.iter_mut().for_each(|x| x.set_name());
        self.players.iter_mut().enumerate().for_each(|(i, x)| x.set_id(i + 1));
    }

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
        self.groups = self.players
            .iter()
            .map(|x| roles::role_map(x.role()))
            .collect();
    }

    fn night(players: &mut RespBoxes, groups: &Vec<Box<dyn RoleGroup>>) {
        groups.iter().for_each(|x| x.night(
            players.iter_mut().collect()    
        ));
        players.iter_mut().for_each(|x| x.send_end());
    }

    fn day(players: &mut RespBoxes) {
        responder::chat(players.iter_mut().collect());
    }

    fn devide(&mut self) -> (RespBoxesMut, RespBoxesMut, RespBoxesMut) {
        let living: Vec<_> = self.players.iter_mut()
            .filter(|x| x.status() == LifeStatus::Alive)
            .collect();
        let (wolves, others): (Vec<_>, Vec<_>) = living
            .into_iter().partition(|x| x.role() == Werewolf);
        let (men, clergies): (Vec<_>, Vec<_>) = others.into_iter()
            .partition(|x| x.role() == Villager);
        (wolves, men, clergies)
    }

    fn is_over(&mut self) -> bool {
        let (wolves, men, clergies) = self.devide();
        let mut msg = String::new();
        if wolves.len() >= men.len() + clergies.len() || clergies.is_empty() {
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
        !msg.is_empty()
        
    }

    fn run(&mut self) {
        loop {
            Self::night(&mut self.players, &self.groups);
            roles::check_death(&mut self.players);
            if self.is_over() { break; }
            Self::day(&mut self.players);
            roles::check_death(&mut self.players);
            if self.is_over() { break; }
        }
    }

    pub fn init(&mut self) {
        self.get_option();
        self.build_connect();
        self.assign_role();
        self.run();
    }
}
