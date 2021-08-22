mod lib;
mod logging;
mod pipeline;

fn main() {
    // Init logging
    logging::init();

    let config = lib::Config::new();

    let pipe;
    match pipeline::Pipeline::new(config.display) {
        Ok(r) => pipe = r,
        Err(e) => {
            panic!("Error! {}", e);
        }
    }

    for i in 0..2 {
        let uri = String::from("rtsp://wowzaec2demo.streamlock.net/vod/mp4:BigBuckBunny_115k.mov");
        let src;
        match pipeline::sources::URISource::new(uri) {
            Ok(v) => src = v,
            Err(e) => {
                panic!("Error! {}", e);
            }
        }

        match pipe.add_source(&src, i) {
            Ok(_) => println!("Source added"),
            Err(e) => {
                panic!("Error! {}", e);
            }
        }
    }

    match pipe.run() {
        Ok(r) => r,
        Err(e) => eprintln!("Error! {}", e),
    }
}
