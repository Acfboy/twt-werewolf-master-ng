use inquire::Text;
pub struct Homepage { }

impl Homepage {
    pub fn main() {
        println!("TwT Werewolf Master {}", env!("CARGO_PKG_VERSION"));
        let res = Text::new("Hello World!").prompt();
        if let Ok(s) = res {
            print!("{}", s);
        }
    }
}