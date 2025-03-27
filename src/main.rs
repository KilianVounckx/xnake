use std::collections::HashMap;

use macroquad::{input::TouchPhase, prelude::*};

const INPUT_QUEUE_CAP: usize = 3;
const TICK_RATE: f32 = 0.2;
const NUM_ROWS: usize = 20;
const NUM_COLS: usize = 20;

#[macroquad::main("Xnake")]
async fn main() {
    rand::srand(miniquad::date::now() as u64);

    let mut game = Game::new();
    loop {
        game = game.update();
        game.draw();
        next_frame().await;
    }
}

#[derive(Debug)]
struct Game {
    time: f32,
    snake: Snake,
    food: Food,
    touches_cache: HashMap<u64, (Vec2, bool)>,
}

#[derive(Debug)]
struct Grid {
    w: f32,
    h: f32,
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

#[derive(Debug)]
struct Snake {
    segments: Vec<IVec2>,
    dir: IVec2,
    input_queue: Vec<IVec2>,
}

#[derive(Debug)]
struct Food {
    typ: FoodType,
    position: IVec2,
}

#[derive(Debug)]
enum FoodType {
    Grow,
}

impl Game {
    fn new() -> Self {
        let touches_cache = HashMap::new();
        let snake = Snake::new(ivec2(NUM_COLS as i32 / 2, NUM_ROWS as i32 / 2), IVec2::X, 3);
        let time = 0.0;
        let food = Food {
            typ: FoodType::Grow,
            position: snake.random_food_location(),
        };

        Self {
            time,
            snake,
            food,
            touches_cache,
        }
    }

    fn update(self) -> Self {
        let Self {
            mut time,
            mut snake,
            mut food,
            mut touches_cache,
        } = self;
        for touch in touches() {
            match touch.phase {
                TouchPhase::Started => {
                    touches_cache.insert(touch.id, (touch.position, false));
                }
                TouchPhase::Moved => {
                    assert!(touches_cache.contains_key(&touch.id));
                    let (position, _) = touches_cache[&touch.id];
                    touches_cache.insert(touch.id, (position, true));
                }
                TouchPhase::Ended => {
                    assert!(touches_cache.contains_key(&touch.id));
                    let (position, has_moved) = touches_cache[&touch.id];
                    if !has_moved {
                        continue;
                    }
                    let dir = touch.position - position;
                    let dir = if dir.x.abs() >= dir.y.abs() {
                        ivec2(dir.x.signum() as i32, 0)
                    } else {
                        ivec2(0, dir.y.signum() as i32)
                    };
                    snake.queue_input(dir);
                    touches_cache.remove(&touch.id);
                }
                TouchPhase::Cancelled => {
                    touches_cache.remove(&touch.id);
                }
                TouchPhase::Stationary => {}
            }
        }

        if is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left) {
            snake.queue_input(IVec2::NEG_X);
        }
        if is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right) {
            snake.queue_input(IVec2::X);
        }
        if is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up) {
            snake.queue_input(IVec2::NEG_Y);
        }
        if is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down) {
            snake.queue_input(IVec2::Y);
        }

        time += get_frame_time();
        while time > TICK_RATE {
            time -= TICK_RATE;
            snake.update();
        }

        if snake.eats(food.position) {
            match food.typ {
                FoodType::Grow => snake.grow(),
            }
            food = Food {
                typ: FoodType::Grow,
                position: snake.random_food_location(),
            };
        }

        if snake.is_outside() {
            return Self::new();
        }

        Self {
            time,
            snake,
            food,
            touches_cache,
        }
    }

    fn draw(&self) {
        clear_background(BLACK);
        let grid = Grid::calculate();
        grid.draw();
        self.food.draw(&grid);
        self.snake.draw(&grid);
    }
}

impl Grid {
    fn calculate() -> Self {
        let size = screen_width().min(screen_height());
        let vmargin = if size == screen_width() {
            (screen_height() - size) / 2.0
        } else {
            0.0
        };
        let (top, bottom) = (vmargin, screen_height() - vmargin);
        let hmargin = if size == screen_height() {
            (screen_width() - size) / 2.0
        } else {
            0.0
        };
        let (left, right) = (hmargin, screen_width() - hmargin);
        let w = size / NUM_COLS as f32;
        let h = size / NUM_ROWS as f32;
        Self {
            w,
            h,
            left,
            right,
            top,
            bottom,
        }
    }

    fn draw(&self) {
        for i in 0..=NUM_ROWS {
            let y = self.top + i as f32 * self.h;
            draw_line(self.left, y, self.right, y, 2.0, WHITE);
        }
        for j in 0..=NUM_COLS {
            let x = self.left + j as f32 * self.w;
            draw_line(x, self.top, x, self.bottom, 2.0, WHITE);
        }
    }
}

impl Snake {
    fn new(pos: IVec2, dir: IVec2, size: i32) -> Self {
        let segments = (0..size).map(|i| pos - i * dir).collect();
        Self {
            segments,
            dir,
            input_queue: vec![],
        }
    }

    fn queue_input(&mut self, dir: IVec2) {
        if self.input_queue.len() < INPUT_QUEUE_CAP && -self.last_input() != dir {
            self.input_queue.push(dir);
        }
    }

    fn last_input(&self) -> IVec2 {
        self.input_queue.last().copied().unwrap_or(self.dir)
    }

    fn random_food_location(&self) -> IVec2 {
        let mut free_positions = (0..NUM_COLS)
            .flat_map(|x| (0..NUM_ROWS).map(move |y| ivec2(x as i32, y as i32)))
            .filter(|v| !self.segments.contains(v));
        let num_free_positions = free_positions.clone().count();
        let index = rand::gen_range(0, num_free_positions);
        free_positions.nth(index).unwrap()
    }

    fn head(&self) -> IVec2 {
        self.segments[0]
    }

    fn head_mut(&mut self) -> &mut IVec2 {
        &mut self.segments[0]
    }

    fn eats(&self, food: IVec2) -> bool {
        self.head() == food
    }

    fn grow(&mut self) {
        let last = *self.segments.last().unwrap();
        self.segments.push(last);
    }

    fn is_outside(&self) -> bool {
        self.head().x < 0
            || self.head().x >= NUM_COLS as i32
            || self.head().y < 0
            || self.head().y >= NUM_ROWS as i32
    }

    fn update(&mut self) {
        if !self.input_queue.is_empty() {
            self.dir = self.input_queue.remove(0);
        }

        {
            let len = self.segments.len();
            self.segments.copy_within(0..len - 1, 1);
        }
        let dir = self.dir;
        *self.head_mut() += dir;
    }

    fn draw(&self, grid: &Grid) {
        for segment in &self.segments {
            let x = grid.left + segment.x as f32 * grid.w;
            let y = grid.top + segment.y as f32 * grid.h;
            draw_rectangle(x, y, grid.w, grid.h, GREEN);
            draw_rectangle_lines(x, y, grid.w, grid.h, 2.0, DARKGREEN);
        }
        draw_circle(
            grid.left
                + self.head().x as f32 * grid.w
                + grid.w / 2.0
                + self.dir.x as f32 * grid.w / 4.0,
            grid.top
                + self.head().y as f32 * grid.h
                + grid.h / 2.0
                + self.dir.y as f32 * grid.w / 4.0,
            grid.w.min(grid.h) / 8.0,
            DARKGREEN,
        );
    }
}

impl Food {
    fn draw(&self, grid: &Grid) {
        let (color, border_color) = match self.typ {
            FoodType::Grow => (RED, RED),
        };
        let x = grid.left + self.position.x as f32 * grid.w;
        let y = grid.top + self.position.y as f32 * grid.h;
        draw_rectangle(x, y, grid.w, grid.h, color);
        draw_rectangle_lines(x, y, grid.w, grid.h, 2.0, border_color);
    }
}
