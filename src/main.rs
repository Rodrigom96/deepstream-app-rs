mod logging;
mod pipeline;
mod pipeline_manager;

#[cfg(test)]
mod test;

fn main() {
    // Init logging
    logging::init();
    let mut manger = match pipeline_manager::PipelineManager::new("config/pipeline_config.yml") {
        Ok(manger) => manger,
        Err(e) => panic!("Error instancing pipeline manager {}", e),
    };

    match manger.run() {
        Ok(r) => r,
        Err(e) => eprintln!("Error! {}", e),
    }
}
