//! ## 基本介绍
//! 谭炜谭狼人杀大师 NG 是[谭炜谭狼人杀](https://twt-tec.github.io/)的 rust 语言重构版本。  
//! 相比于原先的版本，新一代谭炜谭狼人杀具有以下优点：
//! 1. **内存安全**。rust 的所有权和生命周期机制保证了内存安全。
//! 1. **全平台**。用 `std::net::TcpStream` 替代了原来的 winsock，新版本可以在 Windows 以外的平台运行。
//! 1. **代码可读性高**。新版本结构设计合理，实现思路清晰，代码可读性大大提高。
//! 1. **可维护性高**。不同于原先“一镜到底”的实现方式，新版模块划分清晰，将代码拆分到符合逻辑的文件树中。
//! 1. **文档完整**。代码注释充分，文档完整清晰。
//! 1. **支持大语言模型**。在老版本发布时，大语言模型还没有出现，使用的是简陋人机。新版本方便接入符合 OpenAI 格式的大语言模型，人机游戏体验大大提升。
//! 1. **更好的 tui 交互**。新版有进度条和选择列表，提供更好的交互体验。
//! ## 文件结构
//! - [`judger`] 服务器的法官功能。
//!     - [`roles`](judger::roles) 定义角色的行为。
//!         - [`villager`](judger::roles::villager), [`hunter`](judger::roles::hunter), [`werewolf`](judger::roles::werewolf) 角色的实现。
//!         - [`responder`](judger::roles::responder) 回应器，用于回应角色的请求。
//!             - [`debug_bot`](judger::roles::responder::debug_bot) 用于测试的回应器。只会“好人过”和随机投票。
//!             - [`doubao_ai`](judger::roles::responder::doubao_ai) 默认接入豆包大模型 4k 上下文轻量级，也可以接入其它 OpenAI API 格式的大模型。    
//!             - [`human`](judger::roles::responder::human) 和人类用户通信。
//!             - [`widget`](judger::roles::responder::widget) 实现回应器进行投票、讨论的小工具。
//!     - [`log`](judger::log) 生成日志。
//! - [`player`] 玩家客户端。
//!     -[`comm`](player::comm) 玩家客户端通信组件。
//!     - [`villager`](player::villager), [`hunter`](player::hunter), [`werewolf`](player::werewolf) 角色的实现。
//! ## 使用提醒
//! 如果你要接入豆包，需要设置环境变量 `ENDPOING_ID` 和 `API_KEY`。
//! ## 待办清单
//! 1. 实现女巫和预言家。
//! 2. 实现更多角色
//! 3. 加入警长功能。

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
