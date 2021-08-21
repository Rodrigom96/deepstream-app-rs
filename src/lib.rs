use std::env;

pub struct Config {
    pub display: bool,
}

impl Config {
    pub fn new() -> Config {
        let display =  !env::var("DISPLAY").is_err();

        Config {
            display,
        }
    }
}