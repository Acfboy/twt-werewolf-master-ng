use std::collections::HashMap;

use super::LifeStatus::{*};
use super::RespBoxesMut;

pub fn make_votelist(list: &Vec<&mut Box<dyn super::Responder>>) -> Vec<(usize, String)> {
    list.iter().map(|x| {
        (x.get_id(), x.name())
    })
    .collect()
}


/// 进行一次投票。结果格式为 (投票详情字符串，选票指向的玩家的 id)。
fn get_votes(
    voters: &mut RespBoxesMut, 
    list: &Vec<(usize, String)>, 
    msg: String
) -> Vec<(String, usize)> {
    voters.iter_mut()
        .map(|x| x.vote(&msg, list.clone()))
        .collect()
}

/// 进行计数，返回的是 `Vec<(编号, 数量)>`。
fn count_votes(list: Vec<usize>) -> Vec<(usize, usize)> {
    let mut count = HashMap::new();
    list.iter().for_each(|x| { count
        .entry(*x)
        .and_modify(|x| *x += 1)
        .or_insert(1);
    });
    count.into_iter().collect()
}

fn max_votes(count: Vec<(usize, usize)>) -> Vec<usize> {
    let max_num = count.iter().map(|x| x.1).max().unwrap();
    count.into_iter()
        .filter_map(|x| {
            if x.1 == max_num { Some(x.0) }
            else { None }
        })
        .collect()
}

pub fn chat(chater: RespBoxesMut) {
    let (chater, mut dead): (Vec<_>, Vec<_>) = chater.into_iter()
        .partition(|x| x.status() == Alive);
    let mut succ = chater;
    let mut prev: RespBoxesMut = Vec::new();
    succ.iter_mut().for_each(|x| x.send_begin());
    while !succ.is_empty() {
        let mut it = succ.into_iter();
        let speaking = it.next().unwrap();
        speaking.send_begin();
        let words = speaking.rec_text();
        speaking.send_end();
        succ = it.collect();
        dead.iter_mut().for_each(|x| x.send_msg(&words));
        prev.iter_mut().for_each(|x| x.send_msg(&words));
        succ.iter_mut().for_each(|x| x.send_msg(&words));
        prev.push(speaking);
    }
    prev.into_iter().for_each(|x| x.send_end());
}

fn send_detail(voters: &mut RespBoxesMut, s: String) {
    voters.iter_mut().for_each(|x| x.send_msg(&s));
}

/// 投票。如果出现平票，则递归调用自己再次投票。投票结果返回的是用户的 id。
pub fn vote(
    voters: &mut RespBoxesMut, 
    list: Vec<(usize, String)>, 
    msg: String
) -> usize {
    if list.len() == 1 {
        return list[0].0;
    }
    let (detail_str, res): (Vec<_>, Vec<_>) = 
        get_votes(voters, &list, msg).into_iter().unzip();
    let detail = detail_str.join("\n");
    send_detail(voters, detail);
    let count_list = count_votes(res);
    let again_list = max_votes(count_list);
    let cur_res: Vec<_> = list.into_iter()
        .filter(|x| again_list.contains(&x.0))
        .collect();
    vote(voters, cur_res, "请在平票玩家中再次投票。".to_string())
}
