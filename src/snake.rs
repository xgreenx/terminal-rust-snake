use std::io::stdout;
use std::thread;
use std::time::Duration;

use crossterm::{
    cursor::{self},
    event::{KeyCode, KeyEvent},
    terminal::{self, disable_raw_mode, enable_raw_mode},
    ExecutableCommand, Result,
};
use rand::Rng;

use crate::events::{send_events, KeyEventQueue};
use crate::screen_buffer::{Coordinate, GameContent, ScreenBuffer};

const TEXT: &'static str =
    "Hello, my dear Hlib. I hope you are well. Today is your birthday, and I wish you all the best.

I wish good health to you and your family. I hope they will be untouchable by the war as Enchantress from Dota 2.

I wish you to write solid code without weird bugs that consume your time for debugging them. I hope creepers from Minecraft will not hide in the code to explode at a crucial moment.

I wish you to launch the mainnet soon and without any trouble. I hope it will work perfectly and you will be happy with your code's quality and contribution.

I wish you to fully enjoin life.
";
const PADDING: usize = 4;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SnakeGame {
    reveal: bool,
    is_new: bool,
    score: usize,
    screen_buffer: ScreenBuffer,
}

impl SnakeGame {
    pub fn new(reveal: bool, screen_buffer: ScreenBuffer) -> SnakeGame {
        SnakeGame {
            reveal,
            is_new: true,
            score: 0,
            screen_buffer,
        }
    }

    pub fn run(&mut self, target_fps: f64) -> Result<()> {
        let event_queue = KeyEventQueue::new();
        let thread_event_queue = event_queue.clone();

        // launch seperate thread to deal with keyboard input
        thread::spawn(move || send_events(&thread_event_queue));

        let mut stdout = stdout();
        enable_raw_mode()?;
        stdout.execute(cursor::Hide)?;

        stdout.execute(terminal::Clear(terminal::ClearType::All))?;

        let screen_width = self.screen_buffer.height();
        let screen_height = self.screen_buffer.width();

        if self.is_new {
            // clear screen
            self.screen_buffer.set_all(GameContent::Empty);
            self.screen_buffer
                .set_centered_text_at_row(screen_height / 2 - 6, "SNAKE");
            self.screen_buffer
                .set_centered_text_at_row(screen_height / 2 - 4, "ESC to stop");
            self.screen_buffer
                .set_centered_text_at_row(screen_height / 2 + 2, "~ CONTROLS IT by ARROWS ~");

            for n in (0..5).rev() {
                self.screen_buffer
                    .set_centered_text_at_row(screen_height - 2, &format!("Starting in {}", n));
                self.screen_buffer.draw(&mut stdout)?;
                thread::sleep(Duration::from_secs(1));
            }

            self.screen_buffer.set_all(GameContent::Empty);

            if self.reveal {
                self.screen_buffer.set_all(GameContent::Empty);
            } else {
                self.screen_buffer.set_all(GameContent::Food);
            }
            self.is_new = false;
        }

        let mut player = Player::new(
            KeyEvent::from(KeyCode::Left),
            KeyEvent::from(KeyCode::Right),
            KeyEvent::from(KeyCode::Up),
            KeyEvent::from(KeyCode::Down),
        );

        // 0: up, 1: right, 2: down, 3: left
        let mut game_loop_begin = std::time::SystemTime::now();
        let mut game_loop_end = std::time::SystemTime::now();
        let horizontal_target_cycle_time = Duration::from_secs_f64(1.0 / target_fps);
        'outer: loop {
            // ensure constant cycle time of game loop (i.e. constant snake speed)
            let game_loop_runtime = game_loop_end.duration_since(game_loop_begin).unwrap();
            let target_cycle_time = horizontal_target_cycle_time;

            if game_loop_runtime < target_cycle_time {
                thread::sleep(target_cycle_time - game_loop_runtime);
            }

            game_loop_begin = std::time::SystemTime::now();
            if let Some(events) = event_queue.get_all_events() {
                if !events.is_empty() {
                    if !find_matches(
                        &events,
                        &[
                            KeyEvent::from(KeyCode::Esc),
                            KeyEvent::from(KeyCode::Char('q')),
                        ],
                    )
                    .is_empty()
                    {
                        break 'outer;
                    }

                    let event_matches = find_matches(
                        &events,
                        &[
                            player.left_key,
                            player.right_key,
                            player.up_key,
                            player.down_key,
                        ],
                    );

                    if !event_matches.is_empty() {
                        player.update_snake_direction(*event_matches.last().unwrap(), true);
                    }
                }
            }

            let removed_tail = move_snake(&mut player.snake.body_pos, player.snake.direction);
            self.screen_buffer
                .set_at(removed_tail.row, removed_tail.col, GameContent::Empty);

            let head = player.snake.body_pos[0];
            if let GameContent::Food = self.screen_buffer.get_at(head.row, head.col) {
                self.score += 1;

                // grow snake
                player
                    .snake
                    .body_pos
                    .push(*player.snake.body_pos.last().unwrap());
            }

            // check for snake border and snake ego collisions
            if check_border_and_ego_collision(&player.snake.body_pos, screen_width, screen_height) {
                player.snake.body_pos.into_iter().for_each(|coordinate| {
                    self.screen_buffer
                        .set_at(coordinate.row, coordinate.col, GameContent::Empty);
                });
                player.snake = Snake::new_random(screen_height, screen_width);
            }

            // clear, update and draw screen buffer
            add_snake_to_buffer(&mut self.screen_buffer, &player.snake.body_pos);

            self.screen_buffer.add_border(GameContent::Border);
            self.screen_buffer
                .set_centered_text_at_row(0, &format!("Score: {}", self.score));
            self.screen_buffer.fill_with_text(TEXT.to_string(), PADDING);
            self.screen_buffer.draw(&mut stdout)?;

            game_loop_end = std::time::SystemTime::now();
        }
        player.snake.body_pos.into_iter().for_each(|coordinate| {
            self.screen_buffer
                .set_at(coordinate.row, coordinate.col, GameContent::Empty);
        });

        stdout.execute(cursor::Show)?;
        disable_raw_mode()
    }
}

pub fn move_snake(snake: &mut Vec<Coordinate>, snake_direction: Direction) -> Coordinate {
    // add head in new direction
    let new_head = match snake_direction {
        Direction::UP => Coordinate {
            // up
            row: snake[0].row - 1,
            col: snake[0].col,
        },
        Direction::RIGHT => Coordinate {
            // right
            row: snake[0].row,
            col: snake[0].col + 1,
        },
        Direction::DOWN => Coordinate {
            // down
            row: snake[0].row + 1,
            col: snake[0].col,
        },
        Direction::LEFT => Coordinate {
            // left
            row: snake[0].row,
            col: snake[0].col - 1,
        },
    };

    snake.insert(0, new_head);
    // remove tail
    snake.pop().unwrap()
}

pub fn snake_item_collision(snake: &[Coordinate], item: &Coordinate) -> bool {
    let is_collision = snake.iter().position(|&r| r == *item);
    is_collision.is_some()
}

pub fn check_border_and_ego_collision(
    snake_body: &[Coordinate],
    screen_width: usize,
    screen_height: usize,
) -> bool {
    snake_body[0].row == 0
        || snake_body[0].row == screen_height - 1
        || snake_body[0].col == 0
        || snake_body[0].col == screen_width - 1
        || snake_item_collision(&snake_body[1..], &snake_body[0])
}

pub fn find_matches<T: PartialEq + Copy>(look_in: &[T], look_for: &[T]) -> Vec<T> {
    let mut found: Vec<T> = vec![];
    for a in look_for {
        for b in look_in {
            if a == b {
                found.push(*b);
            }
        }
    }
    found
}

#[derive(PartialEq, Clone, Debug)]
pub struct Snake {
    pub body_pos: Vec<Coordinate>,
    // 0: up, 1: right, 2: down, 3: left
    pub direction: Direction,
}

impl Snake {
    pub fn new() -> Snake {
        let snake_body = vec![
            Coordinate { row: 18, col: 10 },
            Coordinate { row: 19, col: 10 },
            Coordinate { row: 20, col: 10 },
        ];
        Snake {
            body_pos: snake_body,
            direction: Direction::UP,
        }
    }

    pub fn new_random(height: usize, width: usize) -> Snake {
        let mut rng = rand::thread_rng();
        let row = rng.gen_range(1, height - 4);
        let col = rng.gen_range(1, width - 1);
        let snake_body = vec![
            Coordinate { row, col },
            Coordinate { row: row + 1, col },
            Coordinate { row: row + 2, col },
        ];
        Snake {
            body_pos: snake_body,
            direction: Direction::UP,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

#[derive(PartialEq, Clone, Debug)]
pub struct Player {
    pub left_key: crossterm::event::KeyEvent,
    pub right_key: crossterm::event::KeyEvent,
    pub up_key: crossterm::event::KeyEvent,
    pub down_key: crossterm::event::KeyEvent,
    pub snake: Snake,
}

impl Player {
    pub fn new(
        left_key: crossterm::event::KeyEvent,
        right_key: crossterm::event::KeyEvent,
        up_key: crossterm::event::KeyEvent,
        down_key: crossterm::event::KeyEvent,
    ) -> Player {
        Player {
            snake: Snake::new(),
            left_key,
            right_key,
            up_key,
            down_key,
        }
    }
    pub fn update_snake_direction(
        &mut self,
        key_event: crossterm::event::KeyEvent,
        is_four_key_steering: bool,
    ) {
        if is_four_key_steering {
            self._update_direction_four_keys(key_event);
        } else {
            self._update_direction_two_keys(key_event);
        }
    }

    fn _update_direction_four_keys(&mut self, key_event: crossterm::event::KeyEvent) {
        if key_event == self.up_key
            && self.snake.direction != Direction::UP
            && self.snake.direction != Direction::DOWN
        {
            self.snake.direction = Direction::UP;
        } else if key_event == self.down_key
            && self.snake.direction != Direction::UP
            && self.snake.direction != Direction::DOWN
        {
            self.snake.direction = Direction::DOWN;
        } else if key_event == self.left_key
            && self.snake.direction != Direction::RIGHT
            && self.snake.direction != Direction::LEFT
        {
            self.snake.direction = Direction::LEFT;
        } else if key_event == self.right_key
            && self.snake.direction != Direction::RIGHT
            && self.snake.direction != Direction::LEFT
        {
            self.snake.direction = Direction::RIGHT;
        }
    }

    fn _update_direction_two_keys(&mut self, key_event: crossterm::event::KeyEvent) {
        let directions_ordered = vec![
            Direction::UP,
            Direction::RIGHT,
            Direction::DOWN,
            Direction::LEFT,
        ];
        let mut current_dir_index = directions_ordered
            .iter()
            .position(|&r| r == self.snake.direction)
            .unwrap() as i64;

        if key_event == self.left_key {
            current_dir_index -= 1;
        } else if key_event == self.right_key {
            current_dir_index += 1;
        }

        current_dir_index = match current_dir_index {
            -1 => 3,
            _ => current_dir_index % 4,
        };

        self.snake.direction = directions_ordered[current_dir_index as usize];
    }
}

pub fn add_snake_to_buffer(screen_buffer: &mut ScreenBuffer, snake: &[Coordinate]) {
    screen_buffer.set_at(snake[0].row, snake[0].col, GameContent::SnakeHead);

    // only use rest of the body
    let snake_body: Vec<&Coordinate> = snake
        .iter()
        .enumerate()
        .filter(|&(i, _)| i != 0)
        .map(|(_, v)| v)
        .collect();

    for coord in snake_body {
        screen_buffer.set_at(coord.row, coord.col, GameContent::SnakeBody);
    }
}
