use chrono::Local;
use env_logger::{fmt::Color, Builder};
use log::{Level, LevelFilter};
use std::io::Write;

pub fn init() {
    Builder::new()
        .format(|buf, record| {
            let mut level_style = buf.style();
            match record.level() {
                Level::Debug => {level_style.set_color(Color::Blue);},
                Level::Info => {level_style.set_color(Color::Cyan);},
                Level::Warn => {level_style.set_color(Color::Yellow);}
                Level::Error => {level_style.set_color(Color::Red);},
                _ => {},
            };

            writeln!(
                buf,
                "{} [{}] {}:{} - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                level_style.value(record.level()),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .filter(None, LevelFilter::Debug)
        .init();
}
