## 简介

谭炜谭狼人杀大师 NG 是[谭炜谭狼人杀](https://twt-tec.github.io/)的 rust 语言重构版本。

相比于原先的版本，新一代谭炜谭狼人杀具有以下优点：
1. **内存安全**。rust 的所有权和生命周期机制保证了内存安全。
1. **全平台**。用 `std::net::TcpStream` 替代了原来的 winsock，新版本可以在 Windows 以外的平台运行。
1. **代码可读性高**。新版本结构设计合理，实现思路清晰，代码可读性大大提高。
1. **可维护性高**。不同于原先“一镜到底”的实现方式，新版模块划分清晰，将代码拆分到符合逻辑的文件树中。
1. **文档完整**。代码注释充分，文档完整清晰。
1. **支持大语言模型**。在老版本发布时，大语言模型还没有出现，使用的是简陋人机。新版本方便接入符合 OpenAI 格式的大语言模型，人机游戏体验大大提升。
1. **更好的 tui 交互**。新版有进度条和选择列表，提供更好的交互体验。

## 使用

克隆代码库
```sh
git clone https://github.com/Acfboy/twt-werewolf-master-ng.git
```

编译
```sh
cd twt-werewolf-master-ng
cargo build --release
```

运行
```
./target/release/twt-werewolf-master-ng
```

如果你要使用豆包大模型，你需要在[火山方舟大模型服务平台](https://www.volcengine.com/docs/82379/)注册并获得接入点和 API Key，并设置相应的环境变量 `ENDPOINT_ID` 和 `API_KEY`。

## 待办清单
1. 实现女巫和预言家。（目前只实现了狼人、村民和猎人）
2. 实现更多角色
3. 加入警长功能。
