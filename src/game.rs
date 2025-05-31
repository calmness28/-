use macroquad::prelude::*;
use crate::position::Position;
use crate::direction::Direction;
use crate::game_state::GameState;
use crate::vec2_pool::Vec2Pool;
use crate::spatial_hash::SpatialHash;
use crate::texture_cache::TextureCache;
use crate::input_buffer::InputBuffer;

pub struct Game {
    snake: Vec<Position>,
    snake_positions: Vec2Pool,
    food: Position,
    direction: Direction,
    grid_width: i32,
    grid_height: i32,
    cell_size: f32,
    score: i32,
    state: GameState,
    last_move_time: f64,
    move_interval: f64,
    high_score: i32,
    animation_progress: f32,

    spatial_hash: SpatialHash,
    texture_cache: TextureCache,
    input_buffer: InputBuffer,

    _max_snake_length: usize,
    _draw_params_cache: Vec<DrawTextureParams>,
}

impl Game {
    pub fn new() -> Self {
        let grid_width = 25;
        let grid_height = 20;
        let cell_size = 25.0;
        let max_snake_length = (grid_width * grid_height) as usize;

        let mut snake = Vec::with_capacity(max_snake_length);
        snake.push(Position { x: grid_width / 2, y: grid_height / 2 });

        let mut snake_positions = Vec2Pool::new(max_snake_length);
        snake_positions.get((grid_width / 2) as f32, (grid_height / 2) as f32);

        let mut game = Game {
            snake,
            snake_positions,
            food: Position { x: 0, y: 0 },
            direction: Direction::Right,
            grid_width,
            grid_height,
            cell_size,
            score: 0,
            state: GameState::Menu,
            last_move_time: 0.0,
            move_interval: 0.12,
            high_score: 0,
            animation_progress: 0.0,

            spatial_hash: SpatialHash::new(1),
            texture_cache: TextureCache::new(),
            input_buffer: InputBuffer::new(3),
            _max_snake_length: max_snake_length,
            _draw_params_cache: Vec::with_capacity(max_snake_length),
        };

        game.spawn_food();
        game
    }

    pub async fn initialize(&mut self) {
        self.texture_cache.load_textures().await;
    }

    fn reset(&mut self) {
        if self.score > self.high_score {
            self.high_score = self.score;
        }

        self.snake.clear();
        self.snake.push(Position {
            x: self.grid_width / 2,
            y: self.grid_height / 2
        });

        self.snake_positions.reset();
        self.snake_positions.get(
            (self.grid_width / 2) as f32,
            (self.grid_height / 2) as f32
        );

        self.direction = Direction::Right;
        self.score = 0;
        self.move_interval = 0.12;
        self.animation_progress = 0.0;
        self.input_buffer.clear();
        self.spawn_food();
        self.state = GameState::Playing;
    }

    fn spawn_food(&mut self) {
        self.update_spatial_hash();

        loop {
            let x = rand::gen_range(0, self.grid_width);
            let y = rand::gen_range(0, self.grid_height);
            let pos = Position { x, y };

            if !self.spatial_hash.contains(pos) {
                self.food = pos;
                break;
            }
        }
    }

    fn update_spatial_hash(&mut self) {
        self.spatial_hash.clear();
        for &pos in &self.snake {
            self.spatial_hash.insert(pos);
        }
    }

    pub fn handle_input(&mut self) {
        if self.state == GameState::Playing {
            let new_direction = if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                Some(Direction::Up)
            } else if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                Some(Direction::Down)
            } else if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
                Some(Direction::Left)
            } else if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
                Some(Direction::Right)
            } else {
                None
            };

            if let Some(dir) = new_direction {
                let valid = match (&self.direction, &dir) {
                    (Direction::Up, Direction::Down) |
                    (Direction::Down, Direction::Up) |
                    (Direction::Left, Direction::Right) |
                    (Direction::Right, Direction::Left) => false,
                    _ => true,
                };

                if valid {
                    if self.input_buffer.commands.is_empty() {
                        self.direction = dir;
                        self.animation_progress = 0.0;
                        self.last_move_time = get_time() - self.move_interval * 0.8;
                    } else {
                        self.input_buffer.add_command(dir);
                    }
                }
            }
        }

        match self.state {
            GameState::Menu => {
                if is_key_pressed(KeyCode::Space) {
                    self.reset();
                }
            }
            GameState::Playing => {
                if is_key_pressed(KeyCode::Escape) {
                    self.state = GameState::Paused;
                }
            }
            GameState::Paused => {
                if is_key_pressed(KeyCode::Escape) {
                    self.state = GameState::Playing;
                }
            }
            GameState::GameOver => {
                if is_key_pressed(KeyCode::Space) {
                    self.reset();
                } else if is_key_pressed(KeyCode::Escape) {
                    self.state = GameState::Menu;
                }
            }
        }
    }

    pub fn update(&mut self) {
        if self.state != GameState::Playing {
            return;
        }

        let current_time = get_time();
        let time_since_last_move = current_time - self.last_move_time;

        self.animation_progress = (time_since_last_move / self.move_interval).min(1.0) as f32;

        if time_since_last_move < self.move_interval {
            return;
        }

        self.last_move_time = current_time;

        if let Some(buffered_direction) = self.input_buffer.get_next_command() {
            let valid = match (&self.direction, &buffered_direction) {
                (Direction::Up, Direction::Down) |
                (Direction::Down, Direction::Up) |
                (Direction::Left, Direction::Right) |
                (Direction::Right, Direction::Left) => false,
                _ => true,
            };

            if valid {
                self.direction = buffered_direction;
            }
        }

        self.animation_progress = 0.0;

        let head = self.snake[self.snake.len() - 1];
        let new_head = match self.direction {
            Direction::Up => Position { x: head.x, y: head.y - 1 },
            Direction::Down => Position { x: head.x, y: head.y + 1 },
            Direction::Left => Position { x: head.x - 1, y: head.y },
            Direction::Right => Position { x: head.x + 1, y: head.y },
        };

        if new_head.x < 0 || new_head.x >= self.grid_width ||
            new_head.y < 0 || new_head.y >= self.grid_height {
            self.state = GameState::GameOver;
            return;
        }

        if self.spatial_hash.contains(new_head) {
            self.state = GameState::GameOver;
            return;
        }

        self.snake.push(new_head);
        self.snake_positions.get(new_head.x as f32, new_head.y as f32);

        if new_head == self.food {
            self.score += 10;
            self.spawn_food();

            if self.move_interval > 0.04 {
                self.move_interval *= 0.97;
            }
        } else {
            self.snake.remove(0);
            self.snake_positions.reset();
            for &pos in &self.snake {
                self.snake_positions.get(pos.x as f32, pos.y as f32);
            }
        }

        self.update_spatial_hash();
    }

    pub fn draw(&self) {
        clear_background(Color::from_rgba(20, 25, 40, 255));

        match self.state {
            GameState::Menu => self.draw_menu(),
            GameState::Playing => self.draw_game(),
            GameState::Paused => {
                self.draw_game();
                self.draw_pause_overlay();
            }
            GameState::GameOver => {
                self.draw_game();
                self.draw_game_over();
            }
        }
    }

    fn draw_menu(&self) {
        let screen_width = screen_width();
        let screen_height = screen_height();

        let title = "🐍 Ilon O'yini 🐍";
        let title_size = 60.0;
        let title_dims = measure_text(title, None, title_size as u16, 1.0);
        draw_text(
            title,
            (screen_width - title_dims.width) / 2.0,
            screen_height / 2.0 - 100.0,
            title_size,
            GREEN,
        );

        let high_score_text = format!("Rekord: {}", self.high_score);
        let instructions = vec![
            "Space - O'yinni boshlash",
            "WASD yoki strelkalar - boshqarish",
            "ESC - pauza",
            "",
            &high_score_text,
            "",
            "⚡ OPTIMAL VERSIYA ⚡",
        ];

        for (i, instruction) in instructions.iter().enumerate() {
            let y = screen_height / 2.0 - 20.0 + i as f32 * 40.0;
            let dims = measure_text(instruction, None, 24, 1.0);
            draw_text(
                instruction,
                (screen_width - dims.width) / 2.0,
                y,
                24.0,
                if instruction.contains("OPTIMAL") { YELLOW } else { WHITE },
            );
        }
    }

    fn draw_game(&self) {
        let offset_x = (screen_width() - self.grid_width as f32 * self.cell_size) / 2.0;
        let offset_y = (screen_height() - self.grid_height as f32 * self.cell_size) / 2.0 + 30.0;

        self.draw_grid(offset_x, offset_y);
        self.draw_snake_batched(offset_x, offset_y);
        self.draw_food(offset_x, offset_y);
        self.draw_ui();
    }

    fn draw_grid(&self, offset_x: f32, offset_y: f32) {
        let grid_color = Color::from_rgba(40, 45, 60, 255);

        for x in 0..=self.grid_width {
            let x_pos = offset_x + x as f32 * self.cell_size;
            draw_line(
                x_pos, offset_y,
                x_pos, offset_y + self.grid_height as f32 * self.cell_size,
                1.0, grid_color,
            );
        }

        for y in 0..=self.grid_height {
            let y_pos = offset_y + y as f32 * self.cell_size;
            draw_line(
                offset_x, y_pos,
                offset_x + self.grid_width as f32 * self.cell_size, y_pos,
                1.0, grid_color,
            );
        }
    }

    fn draw_snake_batched(&self, offset_x: f32, offset_y: f32) {
        let positions = self.snake_positions.get_slice();

        for (i, (_pos, smooth_pos)) in self.snake.iter().zip(positions.iter()).enumerate() {
            let mut current_pos = *smooth_pos;

            if i == self.snake.len() - 1 && self.animation_progress > 0.0 {
                let target_pos = match self.direction {
                    Direction::Up => Vec2::new(current_pos.x, current_pos.y - self.animation_progress),
                    Direction::Down => Vec2::new(current_pos.x, current_pos.y + self.animation_progress),
                    Direction::Left => Vec2::new(current_pos.x - self.animation_progress, current_pos.y),
                    Direction::Right => Vec2::new(current_pos.x + self.animation_progress, current_pos.y),
                };
                current_pos = target_pos;
            }

            let x = offset_x + current_pos.x * self.cell_size + 2.0;
            let y = offset_y + current_pos.y * self.cell_size + 2.0;
            let size = self.cell_size - 4.0;

            if i == self.snake.len() - 1 {
                if let Some(texture) = &self.texture_cache.snake_head {
                    let rotation = match self.direction {
                        Direction::Right => 0.0,
                        Direction::Down => std::f32::consts::PI / 2.0,
                        Direction::Left => std::f32::consts::PI,
                        Direction::Up => -std::f32::consts::PI / 2.0,
                    };

                    draw_texture_ex(
                        texture,
                        x,
                        y,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(Vec2::new(size, size)),
                            rotation,
                            ..Default::default()
                        },
                    );
                } else {
                    draw_rectangle(x, y, size, size, Color::from_rgba(50, 255, 50, 255));
                }
            } else {
                if let Some(texture) = &self.texture_cache.snake_body {
                    let alpha = (200 - (i * 15).min(150)) as u8;
                    draw_texture_ex(
                        texture,
                        x,
                        y,
                        Color::from_rgba(255, 255, 255, alpha),
                        DrawTextureParams {
                            dest_size: Some(Vec2::new(size, size)),
                            ..Default::default()
                        },
                    );
                } else {
                    let alpha = (200 - (i * 15).min(150)) as u8;
                    draw_rectangle(x, y, size, size, Color::from_rgba(0, 200, 0, alpha));
                }
            }
        }
    }

    fn draw_food(&self, offset_x: f32, offset_y: f32) {
        let food_x = offset_x + self.food.x as f32 * self.cell_size + 2.0;
        let food_y = offset_y + self.food.y as f32 * self.cell_size + 2.0;
        let base_size = self.cell_size - 4.0;

        let pulse = (get_time() * 8.0).sin() as f32 * 0.1 + 1.0;
        let food_size = base_size * pulse;
        let offset = (base_size - food_size) / 2.0;

        if let Some(texture) = &self.texture_cache.food {
            draw_texture_ex(
                texture,
                food_x + offset,
                food_y + offset,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(food_size, food_size)),
                    ..Default::default()
                },
            );
        } else {
            draw_rectangle(food_x + offset, food_y + offset, food_size, food_size,
                           Color::from_rgba(255, 100, 100, 255));
        }
    }

    fn draw_ui(&self) {
        draw_text(&format!("Ball: {}", self.score), 20.0, 30.0, 24.0, WHITE);
        draw_text(&format!("Uzunlik: {}", self.snake.len()), 20.0, 60.0, 24.0, WHITE);
        draw_text(&format!("Rekord: {}", self.high_score), screen_width() - 150.0, 30.0, 24.0, YELLOW);

        draw_text(&format!("FPS: {:.0}", get_fps()), screen_width() - 150.0, 60.0, 20.0, GREEN);
        draw_text(&format!("Bufer: {}", self.input_buffer.commands.len()), screen_width() - 150.0, 90.0, 16.0, GRAY);
    }

    fn draw_pause_overlay(&self) {
        let screen_width = screen_width();
        let screen_height = screen_height();

        draw_rectangle(0.0, 0.0, screen_width, screen_height, Color::from_rgba(0, 0, 0, 128));

        let text = "PAUZA";
        let dims = measure_text(text, None, 48, 1.0);
        draw_text(text, (screen_width - dims.width) / 2.0, screen_height / 2.0 - 20.0, 48.0, WHITE);

        let instruction = "ESC - davom etish";
        let dims2 = measure_text(instruction, None, 24, 1.0);
        draw_text(instruction, (screen_width - dims2.width) / 2.0, screen_height / 2.0 + 30.0, 24.0, GRAY);
    }

    fn draw_game_over(&self) {
        let screen_width = screen_width();
        let screen_height = screen_height();

        draw_rectangle(0.0, 0.0, screen_width, screen_height, Color::from_rgba(0, 0, 0, 200));

        let title = "O'YIN TUGADI!";
        let dims = measure_text(title, None, 48, 1.0);
        draw_text(title, (screen_width - dims.width) / 2.0, screen_height / 2.0 - 60.0, 48.0, RED);

        let score_text = &format!("Yakuniy ball: {}", self.score);
        let dims2 = measure_text(score_text, None, 32, 1.0);
        draw_text(score_text, (screen_width - dims2.width) / 2.0, screen_height / 2.0 - 10.0, 32.0, WHITE);

        if self.score == self.high_score && self.score > 0 {
            let record_text = "🎉 YANGI REKORD! 🎉";
            let dims3 = measure_text(record_text, None, 28, 1.0);
            draw_text(record_text, (screen_width - dims3.width) / 2.0, screen_height / 2.0 + 20.0, 28.0, GOLD);
        }

        let instruction = "SPACE - qayta o'ynash | ESC - menuga";
        let dims4 = measure_text(instruction, None, 20, 1.0);
        draw_text(instruction, (screen_width - dims4.width) / 2.0, screen_height / 2.0 + 60.0, 20.0, GRAY);
    }
}