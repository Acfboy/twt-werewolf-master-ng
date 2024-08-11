use super::{responder, DeathReason, LifeStatus, Responder, RoleGroup};
use super::RespBoxesMut;

#[derive(Default)]
pub struct Hunter {}


impl RoleGroup for Hunter {
    fn death(
        &self, 
        mut players: RespBoxesMut,
        dying: &mut Box<dyn Responder>
    ) {
        dying.send_begin();
        dying.send_msg("你死了，请发表遗言。");
        dying.set_status(LifeStatus::Dead);
        let words = dying.rec_text();
        players.iter_mut().for_each(|x| x.send_msg(&words));
        players = players.into_iter()
            .filter(|x| x.status() == LifeStatus::Alive)
            .collect();
        let mut voter = vec![dying];
        let shoot = responder::vote(
            &mut voter, 
            responder::make_votelist(&players), 
            "选择开枪对象".to_string()
        );
        voter[0].send_end();
        players.iter_mut()
            .find(|x| x.get_id() == shoot).unwrap()
            .set_status(LifeStatus::NewDeath(DeathReason::Gun));
        
    }
}
