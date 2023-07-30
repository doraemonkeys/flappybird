use bracket_lib::prelude::*;

enum GameMode {
    Menu,
    Playing,
    End,
}

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
// x ms per frame
const FRAME_DURATION: f32 = 55.0;
// 虚拟重力加速度
const GRAVITY: f32 = 0.3;
// 初始速度
const INITIAL_SPEED: f32 = 1.0;

struct Player {
    x: i32,
    y: i32,
    speed: f32,
}

impl Player {
    fn new(x: i32, y: i32) -> Self {
        Player {
            x,
            y,
            // velocity: 0.0,
            speed: INITIAL_SPEED,
        }
    }
    // 渲染
    fn render(&mut self, ctx: &mut BTerm) {
        ctx.set(
            self.x,
            self.y,
            RGB::named(YELLOW),
            RGB::named(BLACK),
            to_cp437('@'),
        );
    }

    fn flap(&mut self) {
        self.y -= 2;
        self.speed = INITIAL_SPEED;
    }

    fn gravity(&mut self, frame_time: f32) {
        let frame_time = frame_time / 80.0;
        let new_speed = self.speed + GRAVITY * frame_time;
        let len = self.speed * frame_time + 0.5 * GRAVITY * frame_time * frame_time;
        self.y += len as i32;
        if self.y < 0 {
            self.y = 0;
        }
        self.speed = new_speed;
    }
}

struct State {
    mode: GameMode,
    player: Player,
    // frame_time is used to track how long the player has been alive
    frame_time: f32,
    score: i32,
    obstacle: Vec<Obstacle>,
}

impl State {
    fn new() -> Self {
        State {
            mode: GameMode::Menu,
            player: Player::new(40, 25),
            frame_time: 0.0,
            score: 0,
            obstacle: vec![Obstacle::new(SCREEN_WIDTH, 0)],
        }
    }
    fn restart(&mut self) {
        self.mode = GameMode::Playing;
        self.player = Player::new(40, 25);
        self.frame_time = 0.0;
        self.score = 0;
        self.obstacle = vec![Obstacle::new(SCREEN_WIDTH, 0)];
    }

    fn game_over(&mut self, ctx: &mut BTerm) {
        self.mode = GameMode::End;
        self.dead(ctx);
    }

    fn main_menu(&mut self, ctx: &mut BTerm) {
        self.mode = GameMode::Menu;
        ctx.print_centered(5, "Welcome to Flappy Bird");
        ctx.print_centered(8, "(P) Play Game");
        ctx.print_centered(9, "(Q) Quit Game");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn obstacles_move_left(&mut self) {
        for obstacle in self.obstacle.iter_mut() {
            obstacle.move_left();
        }
        if self.obstacle[0].x < 0 {
            self.obstacle.remove(0);
        }
    }

    fn render_obstacles(&mut self, ctx: &mut BTerm) {
        for obstacle in self.obstacle.iter_mut() {
            obstacle.render(ctx);
        }
    }

    fn play(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(NAVY);

        // frame_time_ms is the time since the last frame in milliseconds
        self.frame_time += ctx.frame_time_ms;

        if self.frame_time > FRAME_DURATION {
            self.frame_time = 0.0;
            self.obstacles_move_left();
        }
        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.player.flap();
        }
        self.player.gravity(ctx.frame_time_ms);
        self.player.render(ctx);
        self.render_obstacles(ctx);

        if self.player.y > SCREEN_HEIGHT || self.obstacle.last().unwrap().hit_obstacle(&self.player)
        {
            self.game_over(ctx);
        }

        ctx.print(0, 0, "Press SPACE to flap.");
        ctx.print(0, 1, &format!("Score: {}", self.score));

        if self.player.x > self.obstacle.last().unwrap().x {
            self.score += 1;
            self.obstacle.push(Obstacle::new(SCREEN_WIDTH, self.score));
        }
    }

    fn dead(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        self.mode = GameMode::End;
        ctx.print_centered(5, "You are dead");
        ctx.print_centered(6, &format!("You earned {} points", self.score));
        ctx.print_centered(9, "(P) Play Again");
        ctx.print_centered(10, "(Q) Quit Game ");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }
}

impl GameState for State {
    // tick is called before rendering each frame
    fn tick(&mut self, ctx: &mut BTerm) {
        // clear the screen
        ctx.cls();

        // print the text
        // ctx.print(1, 1, "Hello Bracket World!");

        match self.mode {
            GameMode::Menu => self.main_menu(ctx),
            GameMode::Playing => self.play(ctx),
            GameMode::End => self.game_over(ctx),
        }
    }
}

struct Obstacle {
    // 横坐标
    x: i32,
    // 洞口中心的纵坐标
    gap_y: i32,
    // 洞口的大小
    size: i32,
}

impl Obstacle {
    fn new(x: i32, score: i32) -> Self {
        let mut random = RandomNumberGenerator::new();
        Obstacle {
            x,
            gap_y: random.range(10, 40),
            size: i32::max(2, 20 - score),
        }
    }

    fn render(&mut self, ctx: &mut BTerm) {
        let screen_x = self.x;
        let half_size = self.size / 2;
        // top half
        for y in 0..self.gap_y - half_size {
            ctx.set(
                screen_x,
                y,
                RGB::named(RED),
                RGB::named(BLACK),
                to_cp437('|'),
            );
        }
        // bottom half
        for y in self.gap_y + half_size..SCREEN_HEIGHT {
            ctx.set(
                screen_x,
                y,
                RGB::named(RED),
                RGB::named(BLACK),
                to_cp437('|'),
            );
        }
    }

    fn hit_obstacle(&self, player: &Player) -> bool {
        let half_size = self.size / 2;
        let does_x_match = player.x == self.x;
        let player_above_gap = player.y < self.gap_y - half_size;
        let player_below_gap = player.y > self.gap_y + half_size;
        does_x_match && (player_above_gap || player_below_gap)
    }

    fn move_left(&mut self) {
        self.x -= 1;
    }
}

fn main() -> BError {
    let context = BTermBuilder::simple80x50()
        .with_title("Flappy Bird")
        .build()?;
    main_loop(context, State::new())
}
