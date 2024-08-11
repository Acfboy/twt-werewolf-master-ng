pub struct Villager { }

impl super::Role for Villager {
    fn born(&self, id: u64, username: String)  {
        println!("{}（{} 号）你的角色是 平民。", username, id);
    }
}

impl Villager {
    pub fn new() -> Self { Villager {} }
}