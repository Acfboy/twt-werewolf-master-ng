mod judger;
mod player;
use inquire::Select;
use judger::Judger;
use player::Player;

fn main() {
    println!("TwT Werewolf Master v{}", env!("CARGO_PKG_VERSION"));
    let opt = vec!["服务器", "玩家"];
    let mes = "请选择模式";
    let res = Select::new(mes, opt).prompt().unwrap();
    match res {
        "服务器" => Judger::new().init(),
        "玩家" => Player::new().init(),
        _ => (),
    }
}
