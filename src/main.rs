use std::{collections::HashMap, iter};

use macroquad::{input::TouchPhase, prelude::*};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{Bounded, FromPrimitive, ToPrimitive};
use rand::{ChooseRandom, RandomRange};

const INPUT_QUEUE_CAP: usize = 3;
const INIT_TICK_RATE: f32 = 0.2;
const NUM_ROWS: usize = 20;
const NUM_COLS: usize = 20;
const MAX_FOODS: usize = 10;
const FOOD_DESPAWN_TIME: f32 = 7.0;
const PORTAL_TIME: f32 = 7.0;
const INVISIBLE_TIME: f32 = 7.0;
const DOUBLE_FOOD_TIME: f32 = 4.0;
const SPEED_CHANGE_TIME: f32 = 3.0;

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
    score: u32,
    snakes: Vec<Snake>,
    foods: HashMap<Food, f32>,
    touches_cache: HashMap<u64, (Vec2, bool)>,
    double_food_time_left: f32,
}

#[derive(Debug)]
struct Grid {
    w: f32,
    h: f32,
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
    hmargin: f32,
    vmargin: f32,
}

#[derive(Debug)]
struct Snake {
    time: f32,
    tick_rate: f32,
    segments: Vec<IVec2>,
    dir: IVec2,
    input_queue: Vec<IVec2>,
    portal_time_left: f32,
    invisible_time_left: f32,
    speed_change_time_left: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Food {
    typ: FoodType,
    position: IVec2,
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, FromPrimitive, ToPrimitive,
)]
enum FoodType {
    #[default]
    Grow,
    DoubleFood,
    Cut,
    Slow,
    Fast,
    Reverse,
    Multiply,
    Invisible,
    Portal,
}

impl Game {
    fn new() -> Self {
        let touches_cache = HashMap::new();
        let snakes = vec![Snake::new(
            ivec2(NUM_COLS as i32 / 2, NUM_ROWS as i32 / 2),
            IVec2::X,
            3,
        )];
        let double_food_time_left = 0.0;
        let foods = iter::once((
            Food {
                typ: FoodType::Grow,
                position: random_food_location(snakes.iter(), iter::empty()).unwrap(),
            },
            0.0,
        ))
        .collect();
        let score = 0;

        Self {
            score,
            snakes,
            foods,
            touches_cache,
            double_food_time_left,
        }
    }

    fn update(self) -> Self {
        let Self {
            mut score,
            mut snakes,
            mut foods,
            mut touches_cache,
            mut double_food_time_left,
        } = self;
        let mut inputs = Vec::new();
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
                    inputs.push(dir);
                    touches_cache.remove(&touch.id);
                }
                TouchPhase::Cancelled => {
                    touches_cache.remove(&touch.id);
                }
                TouchPhase::Stationary => {}
            }
        }

        if is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left) {
            inputs.push(IVec2::NEG_X);
        }
        if is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right) {
            inputs.push(IVec2::X);
        }
        if is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up) {
            inputs.push(IVec2::NEG_Y);
        }
        if is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down) {
            inputs.push(IVec2::Y);
        }
        for input in inputs {
            for snake in &mut snakes {
                snake.queue_input(input);
            }
        }

        let delta = get_frame_time();

        double_food_time_left -= delta;
        if double_food_time_left < 0.0 {
            double_food_time_left = 0.0;
        }

        for snake in &mut snakes {
            snake.update(delta);
        }

        let mut foods_to_remove = vec![];
        for (food, food_time) in foods.iter_mut() {
            *food_time -= delta;
            if *food_time < 0.0 && food.typ != FoodType::Grow {
                foods_to_remove.push(*food);
            }
        }

        let mut num_new_foods = 0;
        let mut snakes_to_insert = Vec::new();
        for food in foods.keys().copied() {
            for snake in &mut snakes {
                if !snake.eats(food.position) {
                    continue;
                }
                foods_to_remove.push(food);

                score += food.typ.score();

                match food.typ {
                    FoodType::Grow => {}
                    FoodType::DoubleFood => double_food_time_left = DOUBLE_FOOD_TIME,
                    FoodType::Cut => snake.cut(),
                    FoodType::Slow => snake.speed_up(),
                    FoodType::Fast => snake.slow_down(),
                    FoodType::Reverse => snake.reverse(),
                    FoodType::Multiply => {
                        let neighbors = snake.spawn_neighbors();
                        for neighbor in neighbors {
                            snakes_to_insert.push(neighbor);
                        }
                    }
                    FoodType::Invisible => snake.invisible(),
                    FoodType::Portal => snake.portal(),
                }

                snake.grow();

                num_new_foods += if double_food_time_left > 0.0 { 2 } else { 1 };
            }
        }
        for snake in snakes_to_insert {
            snakes.push(snake);
        }

        for food in foods_to_remove {
            foods.remove(&food);
        }
        for _ in 0..num_new_foods {
            if foods.len() >= MAX_FOODS {
                break;
            }
            let Some(position) =
                random_food_location(snakes.iter(), foods.keys().map(|food| food.position))
            else {
                break;
            };
            let typ = rand::gen_range(FoodType::min_value(), FoodType::max_value());
            foods.insert(Food { typ, position }, FOOD_DESPAWN_TIME);
        }
        if foods.is_empty() {
            if snakes.len() == NUM_ROWS * NUM_COLS {
                return Self::new();
            }

            let position =
                random_food_location(snakes.iter(), foods.keys().map(|food| food.position))
                    .unwrap();
            let typ = FoodType::Grow;
            foods.insert(Food { typ, position }, FOOD_DESPAWN_TIME);
        }
        let mut snakes_to_remove = Vec::new();
        for (index, snake) in snakes.iter().enumerate() {
            if !snake.can_portal() && snake.is_outside() {
                snakes_to_remove.push(index);
                continue;
            }
            for other_snake in &snakes {
                if !snake.is_invisible()
                    && !other_snake.is_invisible()
                    && snake.eats_other(other_snake)
                {
                    snakes_to_remove.push(index);
                }
            }
        }
        for index in snakes_to_remove {
            snakes.swap_remove(index);
        }
        if snakes.is_empty() {
            return Self::new();
        }

        Self {
            score,
            snakes,
            foods,
            touches_cache,
            double_food_time_left,
        }
    }

    fn draw(&self) {
        clear_background(BLACK);
        let grid = Grid::calculate();
        grid.draw();
        for food in self.foods.keys() {
            food.draw(&grid);
        }
        for snake in &self.snakes {
            snake.draw(&grid);
        }

        let score_font_size = if grid.vmargin == 0.0 {
            grid.hmargin / 5.0
        } else {
            grid.vmargin / 3.0
        };
        draw_text(
            &format!("Score: {}", self.score),
            0.0,
            score_font_size,
            score_font_size,
            ORANGE,
        );

        {
            let mut y = score_font_size;
            let font_size = 20.0;
            if self.double_food_time_left > 0.0 {
                y += font_size;
                draw_text(
                    &format!("D: {:.0}", self.double_food_time_left),
                    0.0,
                    y,
                    font_size,
                    ORANGE,
                );
            }
            if self.snakes[0].portal_time_left > 0.0 {
                y += font_size;
                draw_text(
                    &format!("P: {:.0}", self.snakes[0].portal_time_left),
                    0.0,
                    y,
                    font_size,
                    ORANGE,
                );
            }
            if self.snakes[0].invisible_time_left > 0.0 {
                y += font_size;
                draw_text(
                    &format!("I: {:.0}", self.snakes[0].invisible_time_left),
                    0.0,
                    y,
                    font_size,
                    ORANGE,
                );
            }
        }
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
            hmargin,
            vmargin,
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
            time: 0.0,
            tick_rate: INIT_TICK_RATE,
            input_queue: vec![],
            portal_time_left: 0.0,
            invisible_time_left: 0.0,
            speed_change_time_left: 0.0,
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

    fn head(&self) -> IVec2 {
        self.segments[0]
    }

    fn head_mut(&mut self) -> &mut IVec2 {
        &mut self.segments[0]
    }

    fn tail(&self) -> &[IVec2] {
        &self.segments[1..]
    }

    fn eats(&self, food: IVec2) -> bool {
        self.head() == food
    }

    fn eats_other(&self, other: &Self) -> bool {
        other.tail().iter().any(|segment| self.eats(*segment))
    }

    fn grow(&mut self) {
        let last = *self.segments.last().unwrap();
        self.segments.push(last);
    }

    fn cut(&mut self) {
        let len = self.segments.len();
        let new_len = if len % 2 == 0 { len / 2 } else { len / 2 + 1 };
        self.segments.truncate(new_len);
    }

    fn reverse(&mut self) {
        let new_dir = if self.segments.len() >= 2 {
            let last = self.segments[self.segments.len() - 1];
            let almost_last = self.segments[self.segments.len() - 2];
            last - almost_last
        } else {
            -self.dir
        };
        self.segments.reverse();
        self.dir = new_dir;
        self.input_queue.clear();
    }

    fn spawn_neighbors(&self) -> [Self; 2] {
        [IVec2::Y, IVec2::NEG_Y].map(|rot_dir| {
            let offset_dir = self.dir.rotate(rot_dir);
            let head =
                (self.head() + offset_dir * 3).rem_euclid(ivec2(NUM_COLS as i32, NUM_ROWS as i32));
            Self::new(head, self.dir, 3)
        })
    }

    fn speed_up(&mut self) {
        self.tick_rate /= 2.0;
        self.speed_change_time_left = SPEED_CHANGE_TIME;
    }

    fn slow_down(&mut self) {
        self.tick_rate += 0.1;
        self.speed_change_time_left = SPEED_CHANGE_TIME;
    }

    fn portal(&mut self) {
        self.portal_time_left = PORTAL_TIME;
    }

    fn can_portal(&self) -> bool {
        self.portal_time_left > 0.0
    }

    fn invisible(&mut self) {
        self.invisible_time_left = INVISIBLE_TIME;
    }

    fn is_invisible(&self) -> bool {
        self.invisible_time_left > 0.0
    }

    fn is_outside(&self) -> bool {
        self.head().x < 0
            || self.head().x >= NUM_COLS as i32
            || self.head().y < 0
            || self.head().y >= NUM_ROWS as i32
    }

    fn update(&mut self, delta: f32) {
        self.time += delta;
        self.portal_time_left -= delta;
        if self.portal_time_left < 0.0 {
            self.portal_time_left = 0.0;
        }
        self.invisible_time_left -= delta;
        if self.invisible_time_left < 0.0 {
            self.invisible_time_left = 0.0;
        }
        self.speed_change_time_left -= delta;
        if self.speed_change_time_left < 0.0 {
            self.speed_change_time_left = 0.0;
            self.tick_rate = INIT_TICK_RATE;
        }

        while self.time > self.tick_rate {
            self.time -= self.tick_rate;

            if !self.input_queue.is_empty() {
                self.dir = self.input_queue.remove(0);
            }

            {
                let len = self.segments.len();
                self.segments.copy_within(0..len - 1, 1);
            }

            {
                let dir = self.dir;
                let can_portal = self.can_portal();
                let head = self.head_mut();
                *head += dir;
                if can_portal {
                    *head = head.rem_euclid(ivec2(NUM_COLS as i32, NUM_ROWS as i32));
                }
            }
        }
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
            FoodType::DoubleFood => (RED, GREEN),
            FoodType::Cut => (DARKGRAY, GRAY),
            FoodType::Slow => (GREEN, DARKGREEN),
            FoodType::Fast => (GOLD, WHITE),
            FoodType::Reverse => (ORANGE, BLUE),
            FoodType::Multiply => (PURPLE, LIME),
            FoodType::Invisible => (WHITE, LIGHTGRAY),
            FoodType::Portal => (DARKBLUE, GOLD),
        };
        let x = grid.left + self.position.x as f32 * grid.w;
        let y = grid.top + self.position.y as f32 * grid.h;
        draw_rectangle(x, y, grid.w, grid.h, color);
        draw_rectangle_lines(x, y, grid.w, grid.h, 4.0, border_color);
    }
}

impl FoodType {
    fn score(self) -> u32 {
        match self {
            FoodType::Grow => 100,
            FoodType::DoubleFood => 200,
            FoodType::Cut => 500,
            FoodType::Slow => 200,
            FoodType::Fast => 500,
            FoodType::Reverse => 500,
            FoodType::Multiply => 300,
            FoodType::Invisible => 200,
            FoodType::Portal => 200,
        }
    }
}

fn free_locations<'a, I1, I2>(snakes: I1, foods: I2) -> impl Iterator<Item = IVec2>
where
    I1: Iterator<Item = &'a Snake>,
    I2: Iterator<Item = IVec2>,
{
    let snake_positions = snakes.flat_map(|snake| snake.segments.iter()).copied();
    snake_positions.chain(foods)
}

fn random_food_location<'a, I1, I2>(snakes: I1, foods: I2) -> Option<IVec2>
where
    I1: Iterator<Item = &'a Snake>,
    I2: Iterator<Item = IVec2>,
{
    let mut all_positions = free_locations(snakes, foods);
    let free_positions = (0..NUM_COLS)
        .flat_map(|x| (0..NUM_ROWS).map(move |y| ivec2(x as i32, y as i32)))
        .filter(|v| !all_positions.any(|pos| pos == *v))
        .collect::<Vec<_>>();
    free_positions.choose().copied()
}

impl RandomRange for FoodType {
    fn gen_range(low: Self, high: Self) -> Self {
        FoodType::from_u8(u8::gen_range(
            low.to_u8().unwrap(),
            high.to_u8().unwrap() + 1,
        ))
        .unwrap()
    }
    fn gen_range_with_state(state: &rand::RandGenerator, low: Self, high: Self) -> Self {
        FoodType::from_u8(u8::gen_range_with_state(
            state,
            low.to_u8().unwrap(),
            high.to_u8().unwrap() + 1,
        ))
        .unwrap()
    }
}

impl Bounded for FoodType {
    fn min_value() -> Self {
        Self::Grow
    }
    fn max_value() -> Self {
        Self::Portal
    }
}
