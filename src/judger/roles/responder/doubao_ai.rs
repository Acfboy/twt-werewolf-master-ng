use reqwest::Client;
use serde_json::json;
use tokio;

use crate::judger::roles::{Identity, LifeStatus};

use super::Responder;

const INIT_PROMPT: &str = "
狼人杀是一款多人参与的社交推理游戏，通常在夜晚和白天交替进行，玩家通过沟通、推理和策略来达成各自的胜利条件。

游戏通常在所有狼人被驱逐或狼人与村民数量相等时结束。每个角色都有其独特的策略和玩法，玩家需要通过观察、沟通和推理来找出狼人并保护自己的阵营。
以下是狼人、村民和猎人这三个角色的基本规则：

1. 狼人：
   - 狼人是夜间行动的角色，他们的目标是消灭所有村民或者达到狼人数量与剩余村民数量相等的状态。
   - 每个夜晚，狼人们会秘密地聚集并选择一名玩家进行攻击，如果被选中的是村民，则该村民会被狼人杀害。

2. 村民：
   - 村民是游戏的普通角色，他们没有特殊能力，但可以通过白天的讨论和投票来推断狼人的身份。
   - 村民的目标是找出并投票驱逐狼人，以保护自己不被狼人杀害。

3. 猎人：
   - 猎人是具有特殊能力的村民。如果猎人被狼人杀害或被村民投票驱逐，他们可以在临死前选择一名玩家作为自己的“遗言”，该玩家随即被猎人“射杀”。
   - 猎人也可以选择不使用这个能力，直到游戏结束。

现在，你要去参加谭炜谭狼人杀比赛，谭炜谭狼人杀只有以上三个角色。接下来我会告诉你一些信息，这些消息有的会告诉你你的角色，有的\
会告诉你其它人的发言，有的会告诉你一次投票的结果，还有一些会要求你进行投票或者发言。在轮流发言时，\
如果我要求你“输出你被要求回答的”，则输出你的发言内容，若是死亡后被要求回答，则输出你的遗言内容；\
当你被要求投票时，直接输出相应的编号，不要输出多余内容。
现在，游戏开始。";
const CHAT_PAROMPT: &str = "现在轮到你发言。";
const VOTE_PROMPT: &str = "请在之后的候选名单中选择你想要选择的玩家，并直接输出编号。
比如，如果你想选择“舔一舔（1 号）”，请直接说“1”，而不是“我想选择 1 号”之类的，
如果你想选择“twt（2 号）”，直接说“2”，而不是“好的，我选择 2 号”之类的";

pub struct Doubao {
    prompts: Vec<serde_json::Value>,
    id: usize,
    status: LifeStatus,
    role: Identity,
    res: String,
    username: String,
    tokens: (u64, u64),
}

impl Doubao {
    #[allow(dead_code)]
    pub fn new() -> Self {
        let init_prompt = json!({
            "role": "system",
            "content": INIT_PROMPT,
        });
        Self {
            prompts: vec![init_prompt],
            id: 0,
            status: LifeStatus::Alive,
            role: Identity::Raw,
            res: String::new(),
            username: String::new(),
            tokens: (0, 0)
        }
    }

    fn push(&mut self, msg: String) {
        self.prompts.push(json!({
            "role": "system",
            "content": msg,
        }));
    }

    /// 向大模型 api 请求并获得回应。
    async fn async_get_res(&mut self) -> Result<(), Box<dyn std::error::Error>>  {
        let url = "https://ark.cn-beijing.volces.com/api/v3/chat/completions";
        let client = Client::new();
        let payload = json!({
            "model": std::env::var("ENDPOINT_ID").unwrap(),
            "messages": self.prompts.clone(),
        });
    
        let payload_str = payload.to_string();
        let authorization_header = format!("Bearer {}", std::env::var("API_KEY").unwrap());
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(reqwest::header::CONTENT_TYPE, reqwest::header::HeaderValue::from_static("application/json"));
        headers.insert(reqwest::header::AUTHORIZATION, reqwest::header::HeaderValue::from_str(&authorization_header)?);
    
        let resp = client
            .post(url)
            .headers(headers)
            .body(payload_str)
            .send()
            .await?
            .text()
            .await?;
        let res: serde_json::Value = serde_json::from_str(&resp).unwrap();
        self.res = res["choices"][0]["message"]["content"].as_str().unwrap().to_string();
        // #[cfg(debug_assertions)]
        // println!("ai response: {}", resp);
        let out_tokens = res["usage"]["completion_tokens"].as_u64().unwrap();
        let inp_tokens = res["usage"]["prompt_tokens"].as_u64().unwrap();
        self.tokens.0 += inp_tokens;
        self.tokens.1 += out_tokens;
        Ok(())
    }

    
    /// 发送之前的所有信息提供给大模型并获得回复，不懂异步，直接阻塞。
    fn get_res(&mut self) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            self.async_get_res().await.unwrap();
        });
    }
}

impl Responder for Doubao{
    fn send(&mut self, _msg: &str) {}

    fn rec(&mut self) -> String { 
        panic!("this is ai");
    }

    fn send_number(&mut self, _x: usize) {}

    fn rec_number(&mut self) -> usize {
        panic!("this is ai");
    }

    fn send_begin(&mut self) {}

    fn send_end(&mut self) {}

    fn send_msg(&mut self, msg: &str) {
        self.send(&format!("m{}", msg));
    }

    
    fn rec_text(&mut self) -> String {
        self.prompts.push(json!({
            "role": "user",
            "content": CHAT_PAROMPT,
        }));
        self.get_res();
        self.res.clone()
    }
    
    fn send_json(&mut self, _jstr: &str) { 
        panic!("this is ai")
    }

    fn vote(&mut self, msg: &str, list: Vec<(usize, String)>) -> (String, usize) {
        let (id, names): (Vec<_>, Vec<_>) = list.into_iter().unzip();
        let list = "候选名单 ".to_string() + &names.join(" ");
        self.push(msg.to_string());
        self.push(VOTE_PROMPT.to_string());
        self.prompts.push(json!({
            "role": "user",
            "content": list,
        }));
        self.get_res();
        let num_str: String = self.res.chars().filter(|x| x.is_ascii_digit()).collect();
        let tar = num_str.parse::<usize>().unwrap();
        let mut tar_name = "ERROR";
        for c in 0..id.len() {
            if id[c] == tar {
                tar_name = &names[c];
                break;
            }
        }
        (format!("{} -> {}", self.name(), tar_name), tar)
    }

    fn role(&self) -> Identity {
        self.role.clone()
    }

    fn set_role(&mut self, r: Identity) {
        self.role = r;
    }

    fn status(&self) -> LifeStatus {
        self.status
    }

    fn set_status(&mut self, s: LifeStatus) {
        self.status = s;
    }

    fn set_name(&mut self) {
        self.prompts.push(json!({
            "role": "user",
            "content": "现在先给自己起一个好听的昵称，注意，你需要直接输出昵称。\
             比如，如果你给自己起的昵称是“舔一舔”，不要输出“我叫舔一舔”“我的昵称是舔一舔”之类的，直接输出“舔一舔”，不要包含其它内容",
        }));
        self.get_res();
        self.username = self.res.clone();
    }

    fn name(&self) -> String { 
        format!("{}（{} 号）", self.username, self.id)
    }

    fn set_id(&mut self, id: usize) {
        self.id = id;
    }

    fn get_id(&self) -> usize { 
        self.id
    }

    fn coutinue_game(&mut self) {}

    fn cost(&self) -> (u64, u64) {
        self.tokens
    }

}

