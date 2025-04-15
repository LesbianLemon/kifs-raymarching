use kifs_raymarching::graphics;

fn main() {
    let options = graphics::GraphicsStateOptions::default();

    pollster::block_on(graphics::run(options));
}
