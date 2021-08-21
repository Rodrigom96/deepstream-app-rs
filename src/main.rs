use std::process;

mod lib;
use lib::*;

#[path = "pipeline/pipeline.rs"]
mod pipe;


fn main() {
    let config = Config::new();

    let pipeline;
    match pipe::Pipeline::new(config.display) {
        Ok(r) => pipeline = r,
        Err(e) => {
            eprintln!("Error! {}", e);
            process::exit(1);
        } 
    }

    match pipeline.run() {
        Ok(r) => r,
        Err(e) => eprintln!("Error! {}", e),
    }

    println!("Rust says: Hello, world!");
}
