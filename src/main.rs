mod game;

fn main() {
    let mut siv = cursive::default();
    game::start_menu(&mut siv);
    siv.run();
}
