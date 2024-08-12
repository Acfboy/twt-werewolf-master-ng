//! 定义角色组特型。也包含了角色相关枚举和角色死亡检查。

use std::fmt;

pub mod responder;
pub use responder::Responder;

pub mod villager;
pub mod werewolf;
pub mod hunter;

use super::log::Log;
use super::RespBoxes;
use super::RespBoxesMut;

/// 死亡原因。有的死因有特殊效果，比如女巫毒药。
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum DeathReason {
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

/// 把身份枚举转换成文字名称
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

/// 把身份转换成对应 [`RoleGroup`] 对象。
pub fn role_map(r: Identity) -> Box<dyn RoleGroup> {
    match r {
        Identity::Villager => Box::new(villager::Villager::default()),
        Identity::Werewolf => Box::new(werewolf::Werewolf::default()),
        Identity::Hunter => Box::new(hunter::Hunter::default()),
        Identity::Raw => panic!("role error"),
    }
}

/// 找出死亡角色，递归地处理角色死亡。注意，这里使用了新的 [`RoleGroup`] 对象来处理，而不是原来的。
pub fn check_death(players: &mut RespBoxes, log: &mut Log) {
    println!("--CHECKING DEATH--");
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
    group.death(others, dying, log);
    check_death(players, log);
}


/// 实现角色行为的特型。注意，服务器死亡判断时创建了新对象，谨慎在角色组对象中存储数据。
pub trait RoleGroup {
    #[allow(dead_code)]
    fn day(&self, _players: RespBoxesMut) {
    }

    fn night(&self, _players: RespBoxesMut, _log: &mut Log) {}

    /// 角色一个个死，死亡判断被一个个传给每个角色组。
    /// - 遗言需要广播给所有人，players 需要传入所有人。
    /// - dying 是需要进行死亡判断的玩家。
    fn death(
        &self, 
        mut players: RespBoxesMut, 
        dying: &mut Box<dyn super::Responder>,
        log: &mut Log
    ) {
        players.iter_mut()
            .for_each(|x| x.send_msg(&format!("{} 死了。", dying.name())));
        dying.send_begin();
        dying.send_msg("你死了，请发表遗言。");
        dying.set_status(LifeStatus::Dead);
        let words = format!("{}的遗言: {}", dying.name(), dying.rec_text());
        log.write(&words);
        players.into_iter().for_each(|x| x.send_msg(&words));
    }
}
