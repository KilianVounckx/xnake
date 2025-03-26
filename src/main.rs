use macroquad::prelude::*;

const INPUT_QUEUE_CAP: usize = 3;
const TICK_RATE: f32 = 0.3;
const NUM_ROWS: usize = 20;
const NUM_COLS: usize = 20;

#[macroquad::main("Xnake")]
async fn main() {
    let mut snake = Snake::new(ivec2(NUM_COLS as i32 / 2, NUM_ROWS as i32 / 2), IVec2::X, 3);
    let mut time = 0.0;
    loop {
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

        clear_background(BLACK);

        let grid = Grid::calculate();
        grid.draw();
        snake.draw(&grid);

        next_frame().await;
    }
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

    fn update(&mut self) {
        if !self.input_queue.is_empty() {
            self.dir = self.input_queue.remove(0);
        }

        {
            let len = self.segments.len();
            self.segments.copy_within(0..len - 1, 1);
        }
        self.segments[0] += self.dir;
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
                + self.segments[0].x as f32 * grid.w
                + grid.w / 2.0
                + self.dir.x as f32 * grid.w / 4.0,
            grid.top
                + self.segments[0].y as f32 * grid.h
                + grid.h / 2.0
                + self.dir.y as f32 * grid.w / 4.0,
            grid.w.min(grid.h) / 8.0,
            DARKGREEN,
        );
    }
}
