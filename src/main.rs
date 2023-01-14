extern crate clap;

use clap::{App, Arg};
use std::{fs, io};

mod events;
mod screen_buffer;
mod snake;

use crate::screen_buffer::{GameContent, ScreenBuffer};
use crossterm::Result;
use snake::SnakeGame;

const STATE_FILE: &str = "state.dump";

fn main() -> Result<()> {
    let matches = App::new("snake")
        .version("0.1.0")
        .author("Author: Green")
        .about("Almost a classic snake game for your terminal. You need to eat black squares, \
        and you will reveal something. The game saves the state on exit and loads on start(if you \
        didn't set the '--new' flag). You don't need to be afraid of dying because you can't die=) \
        The game will continue but with a shorter snake. If the default speed is to hight or low for \
        you, then you can change is with the '--speed {number of fps}' flag. If you are tired of \
        the game and want to see the final result, you can specify the '--reveal' flag.")
        .arg(
            Arg::with_name("reveal")
                .short("r")
                .long("reveal")
                .help("reveal the message without game")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("new")
                .short("n")
                .long("new")
                .help("starts a new game")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("speed")
                .short("s")
                .long("speed")
                .help("the speed of the game in fps")
                .takes_value(true)
                .default_value("30"),
        )
        .get_matches();

    let reveal = matches.is_present("reveal");
    let target_fps: f64 = matches
        .value_of("speed")
        .expect("Missed value for speed")
        .parse()
        .expect("Can't parse the speed value");

    let mut game = if matches.is_present("new") {
        new_game(reveal)
    } else {
        load_state().unwrap_or_else(|_| new_game(reveal))
    };

    game.run(target_fps)?;
    let bytes = serde_json::to_string(&game).expect("Can't decode the state");
    if let Err(err) = fs::write(STATE_FILE, bytes) {
        println!("\n Can't save the state {}", err);
    }

    Ok(())
}

fn load_state() -> io::Result<SnakeGame> {
    let bytes = fs::read_to_string(STATE_FILE)?;
    let game = serde_json::from_str(bytes.as_str())?;
    Ok(game)
}

fn new_game(reveal: bool) -> SnakeGame {
    let screen_height = 40;
    let screen_width = 40;
    let screen_buffer = ScreenBuffer::new(screen_width, screen_height, GameContent::Empty);
    SnakeGame::new(reveal, screen_buffer)
}
