use super::{LifeStatus, RoleGroup, Identity, responder, DeathReason};
use super::RespBoxesMut;
#[derive(Default)]
pub struct Werewolf {}

impl RoleGroup for Werewolf {
    fn night(&self, mut players: RespBoxesMut) {
        players = players.into_iter()
            .filter(|x| x.status() == LifeStatus::Alive).collect();
        let vote_list = responder::make_votelist(&players);
        let (mut wolves, others): (Vec<_>, Vec<_>) = players
            .into_iter()
            .partition(|x| x.role() == Identity::Werewolf);
        wolves.iter_mut().for_each(|x| x.send_begin());
        let to_kill = responder::vote(&mut wolves, vote_list, "选出要刀的人。".to_string());
        wolves.iter_mut().for_each(|x| x.send_end());
        others.into_iter()
            .chain(wolves.into_iter())
            .find(|x| x.get_id() == to_kill)
            .unwrap()
            .set_status(LifeStatus::NewDeath(DeathReason::Normal));
    }
}