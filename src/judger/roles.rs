use std::fmt;

pub mod responder;
pub use responder::Responder;

mod villager;
mod werewolf;
mod hunter;

use super::RespBoxes;
use super::RespBoxesMut;

#[derive(PartialEq, Eq, Clone, Copy)]
enum DeathReason {
    Normal,
    Gun,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum LifeStatus {
    Alive,
    Dead,
    NewDeath(DeathReason),
}

#[derive(Clone, PartialEq, Eq)]
pub enum Identity {
    Villager,
    Werewolf,
    Hunter,
    Raw,
}

impl fmt::Display for Identity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Identity::Villager => write!(f, "平民"),
            Identity::Werewolf => write!(f, "狼人"),
            Identity::Hunter => write!(f, "猎人"),
            Identity::Raw => write!(f, "Raw"),
        }
    }
}

pub fn role_map(r: Identity) -> Box<dyn RoleGroup> {
    match r {
        Identity::Villager => Box::new(villager::Villager::default()),
        Identity::Werewolf => Box::new(werewolf::Werewolf::default()),
        Identity::Hunter => Box::new(hunter::Hunter::default()),
        Identity::Raw => panic!("role error"),
    }
}

pub fn check_death(players: &mut RespBoxes) {
    let (gone, mut others): (Vec<_>, Vec<_>) = 
        players.iter_mut()
        .partition(|x| matches!(x.status(), LifeStatus::NewDeath(_)));
    let mut iter_gone = gone.into_iter();
    let dying = iter_gone.next();
    if let None = dying { 
        players.iter_mut().for_each(|x| x.send_end());
        return; 
    }
    let dying = dying.unwrap();
    let group = role_map(dying.role());
    others = others.into_iter().chain(iter_gone).collect();
    group.death(others, dying);
    check_death(players);
}

pub trait RoleGroup {
    fn day(&self, players: RespBoxesMut) {}

    fn night(&self, players: RespBoxesMut) {}

    /// 角色请一个个死，死亡判断由 Basic::chat 处理后一个个传给每个角色组。
    /// - 遗言需要广播给所有人，players 需要传入所有人。
    /// - dying 是需要进行死亡判断的玩家。
    fn death(
        &self, 
        mut players: RespBoxesMut, 
        dying: &mut Box<dyn super::Responder>
    ) {
        players.iter_mut()
            .for_each(|x| x.send_msg(&format!("{} 死了。", dying.name())));
        dying.send_begin();
        dying.send_msg("你死了，请发表遗言。");
        dying.set_status(LifeStatus::Dead);
        let words = format!("{}: {}", dying.name(), dying.rec_text());
        dying.send_end();
        players.into_iter().for_each(|x| x.send_msg(&words));
    }
}
