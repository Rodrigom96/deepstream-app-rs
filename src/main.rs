use std::process;

mod lib;
use lib::*;

mod pipeline;

fn main() {
    let config = Config::new();

    let pipe;
    match pipeline::Pipeline::new(config.display) {
        Ok(r) => pipe = r,
        Err(e) => {
            eprintln!("Error! {}", e);
            process::exit(1);
        }
    }

    for i in 0..2 {
        let uri = String::from("rtsp://wowzaec2demo.streamlock.net/vod/mp4:BigBuckBunny_115k.mov");
        let src;
        match pipeline::sources::URISource::new(uri) {
            Ok(v) => src = v,
            Err(e) => {
                eprintln!("Error! {}", e);
                process::exit(1);
            }
        }

        match pipe.add_source(&src, i) {
            Ok(_) => println!("Source added"),
            Err(e) => {
                eprintln!("Error! {}", e);
                process::exit(1);
            }
        }
    }

    match pipe.run() {
        Ok(r) => r,
        Err(e) => eprintln!("Error! {}", e),
    }

    println!("Rust says: Hello, world!");
}
