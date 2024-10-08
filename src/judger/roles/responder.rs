use super::Identity::{self, *};
use super::LifeStatus::{self, *};
pub mod human;
pub mod widget;
pub mod doubao_ai;
pub mod debug_bot;
pub use widget::*;
use super::RespBoxesMut;

/// 定义响应器特型。  
/// 响应器特型里实现了服务器需要的各种和玩家交互的方法。
pub trait Responder {
    /// 发送原始信息。ai 玩家不需要原始信息。
    fn send(&mut self, _msg: &str) { }

    /// 接收一条原始信息。ai 玩家不需要接收原始信息。
    fn rec(&mut self) -> String { String::new() }

    /// 默认实现发送数字，开头以 `n` 标记。
    fn send_number(&mut self, x: usize) {
        self.send(&format!("n{}", x.to_string()));
    }

    /// 接收一个数字。
    fn rec_number(&mut self) -> usize {
        let msg = self.rec();
        assert_eq!(msg.chars().nth(0).unwrap(), 'n');
        msg.chars().skip(1).collect::<String>().parse().unwrap()
    }

    /// 表示和一个应答者通话的开始。
    fn send_begin(&mut self) {
        self.send("y");
    }

    /// 表示一次等待的结束作为结束。
    fn send_end(&mut self) {
        self.send("o");
    }

    /// 用于发送直接显示在客户端上的消息，函数会在开头以 `m` 标记。
    fn send_msg(&mut self, msg: &str) {
        self.send(&format!("m{}", msg));
    }

    /// 接收文本。文本会被以 `t` 开头标记。
    fn rec_text(&mut self) -> String {
        let msg = self.rec();
        assert_eq!(msg.chars().nth(0).unwrap(), 't');
        msg.chars().skip(1).collect::<String>()
    }
    
    /// 传入原始 json 字符串，会被标记为 `j` 发送。
    fn send_json(&mut self, jstr: &str) {
        self.send(&format!("j{}", jstr));
    }

    /// 投票。返回 (详情字符串, 选票指向的 id)。  
    /// 注意投票需要开始信号，因为平票多次投票中，玩家只多次响应单次投票，而把详细信息当作普通信息显示。
    fn vote(&mut self, msg: &str, list: Vec<(usize, String)>) -> (String, usize) {
        let (id, names): (Vec<_>, Vec<_>) = list.into_iter().unzip();
        let json = serde_json::to_string(&names).unwrap();
        self.send_begin();
        self.send_msg(msg);
        self.send_json(&json);
        let x = self.rec_number();
        (format!("{} -> {}", self.name(), names[x]), id[x])
    }

    fn role(&self) -> Identity {
        Raw
    }

    fn set_role(&mut self, _r: Identity) {}

    fn status(&self) -> LifeStatus {
        Alive
    }

    fn set_status(&mut self, _s: LifeStatus) {}

    fn set_name(&mut self) {}

    fn name(&self) -> String { String::new() }

    fn set_id(&mut self, _id: usize) {}

    fn get_id(&self) -> usize { 0 }

    /// 在玩家端游戏结束判定时发送继续游戏。其实只是发送一个结束信号。
    fn coutinue_game(&mut self) {
        self.send_end();
    }

    /// 在玩家端结束游戏判定时发送游戏结束信息。其实只是发送一条获胜信息。
    fn game_over(&mut self, msg: String) {
        self.send_msg(&msg);
    }

    fn cost(&self) -> (u64, u64) { (0, 0) }
}