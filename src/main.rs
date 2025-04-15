use kifs_raymarching::graphics;

async fn start() {
    let options = graphics::GraphicsStateOptions::default();

    match graphics::run(options).await {
        Ok(()) => println!("Exiting program..."),
        Err(err) => log::error!("{err}"),
    }
}

fn main() {
    pollster::block_on(start());
}
