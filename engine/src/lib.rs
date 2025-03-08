pub mod engine_impl {
    pub fn start_game() {
        println!("Engine is running the game!");
    }
}

pub use engine_impl::start_game;