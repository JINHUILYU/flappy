use bracket_lib::prelude::*;

// 定义游戏的模式
enum GameMode {
    Menu,
    Playing,
    GameOver,
}

const SCREEN_WIDTH: i32 = 80;     // 屏幕宽度
const SCREEN_HEIGHT: i32 = 50;    // 屏幕高度
const FRAME_DURATION: f32 = 75.0; // 每帧的间隔

// 定义玩家的结构体
struct Player {
    x: i32,        // 世界空间横坐标
    y: i32,        // 世界空间纵坐标
    velocity: f32, // 垂直方向的速度
}

impl Player {
    // 创建一个玩家对象
    fn new(x: i32, y: i32) -> Self {
        Self {
            x,             // 世界空间横坐标
            y,             // 世界空间纵坐标
            velocity: 0.0, // 垂直方向的速度
        }
    }

    // 渲染玩家
    fn render(&self, ctx: &mut BTerm) {
        ctx.set(0, self.y, YELLOW, BLACK, to_cp437('@'));
    }

    // 处理重力和移动：随着时间的推移，玩家的速度会增加（模拟重力），并更新玩家位置
    fn gravity_and_move(&mut self) {
        // 限制最大速度
        if self.velocity < 2.0 {
            self.velocity += 0.2;
        }
        // 每调用一次，x 增加 1，相当于向前移动
        self.x += 1;
        // y 位置根据速度变化
        self.y += self.velocity as i32;

        // 防止玩家穿过上边界
        if self.y < 0 {
            self.y = 0;
        }
    }

    // 处理玩家的跳跃：当玩家按下空格键时，速度设置为 -2.0，模拟向上跳跃
    fn flap(&mut self) {
        self.velocity = -2.0;
    }
}

// 游戏状态
struct State {
    player: Player,
    frame_time: f32,
    mode: GameMode,
    obstacle: Obstacle,
    score: i32,
}

impl State {
    // 创建一个新的游戏状态
    fn new() -> Self {
        Self {
            // 初始位置
            player: Player::new(5, 25),
            frame_time: 0.0,
            // 初始模式
            mode: GameMode::Menu,
            // 初始化第一个障碍物
            obstacle: Obstacle::new(SCREEN_WIDTH, 0),
            // 初始化分数
            score: 0,
        }
    }

    // 主菜单
    fn main_menu(&mut self, ctx: &mut BTerm) {
        ctx.cls(); // 清除屏幕
        ctx.print_centered(5, "Welcome to Flappy Dragon");
        ctx.print_centered(8, "Press [Enter] to start");
        ctx.print_centered(9, "Press [Esc] to quit");

        // 根据按键输入来决定游戏的状态
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::Return => self.restart(),
                VirtualKeyCode::Escape => ctx.quitting = true,
                _ => {}
            }
        }
    }

    // 游戏进行中
    fn play(&mut self, ctx: &mut BTerm) {
        ctx.cls(); // 清除屏幕

        // 帧时间累加，如果超过设定的帧间隔，则更新玩家位置
        self.frame_time += ctx.frame_time_ms;
        if self.frame_time > FRAME_DURATION {
            self.frame_time = 0.0;
            // 更新玩家位置
            self.player.gravity_and_move();
        }

        // flag：如果按下空格键，则调用玩家的跳跃函数
        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.player.flap();
        }

        // 渲染玩家和障碍物
        self.player.render(ctx);
        // 左上角打印相关信息
        ctx.print(0, 0, "Press Space to Flap!");
        ctx.print(0, 1, &format!("Score: {}", self.score));

        // 渲染障碍物
        self.obstacle.render(ctx, self.player.x);

        // 当玩家越过障碍物时，更新分数并生成新的障碍物
        if self.player.x > self.obstacle.x {
            self.score += 1;
            self.obstacle = Obstacle::new(self.player.x + SCREEN_WIDTH, self.score);
        }

        // 玩家位于屏幕外或碰到障碍物
        if self.player.y > SCREEN_HEIGHT || self.obstacle.hit_obstacle(&self.player) {
            self.mode = GameMode::GameOver;
        }
    }

    // 重启游戏
    fn restart(&mut self) {
        self.player = Player::new(5, 25);
        self.frame_time = 0.0;
        self.mode = GameMode::Playing;
        self.obstacle = Obstacle::new(SCREEN_WIDTH, 0);
        self.score = 0;
    }

    // 游戏结束
    fn game_over(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "Game Over");
        ctx.print_centered(6, &format!("You earn {} points.", self.score));
        ctx.print_centered(8, "Press [Enter] to start again");
        ctx.print_centered(9, "Press [Esc] to quit");

        // 根据玩家在游戏结束界面按下的键，决定后续逻辑
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::Return => self.restart(),
                VirtualKeyCode::Escape => ctx.quitting = true,
                _ => {}
            }
        }
    }
}

// 实现 bracket-lib 的 GameState trait
impl GameState for State {
    // 每一帧都会调用 tick 方法，用于根据当前模式渲染不同的内容
    fn tick(&mut self, ctx: &mut BTerm) {
        match self.mode {
            GameMode::Menu => self.main_menu(ctx),
            GameMode::Playing => self.play(ctx),
            GameMode::GameOver => self.game_over(ctx),
        }
    }
}

// 障碍物结构体
struct Obstacle {
    x: i32,     // 世界空间横坐标
    gap_y: i32, // 世界空间纵坐标
    size: i32,  // 缺口大小
}

impl Obstacle {
    // 创建一个新的障碍物
    fn new(x: i32, score: i32) -> Self {
        let mut random = RandomNumberGenerator::new();
        Obstacle {
            x,
            // 随机生成缺口位置
            gap_y: random.range(10, 40),
            // 根据分数计算缺口大小，分数越高，缺口越小（最小为 2）
            size: i32::max(2, 20 - score),
        }
    }

    // 渲染障碍物
    fn render(&self, ctx: &mut BTerm, player_x: i32) {
        let screen_x = self.x - player_x; // 屏幕空间
        let half_size = self.size / 2;    // 缺口大小的一半

        // 渲染障碍物的上半部分
        for y in 0..self.gap_y - half_size {
            ctx.set(screen_x, y, RED, BLACK, to_cp437('|'));
        }
        // 渲染障碍物的下半部分
        for y in self.gap_y + half_size..SCREEN_HEIGHT {
            ctx.set(screen_x, y, RED, BLACK, to_cp437('|'));
        }
    }

    // 判断玩家是否碰到障碍物
    fn hit_obstacle(&self, player: &Player) -> bool {
        let half_size = self.size / 2;
        let does_x_match = self.x == player.x;
        let player_above_gap = player.y < self.gap_y - half_size;
        let player_blow_gap = player.y > self.gap_y + half_size;
        does_x_match && (player_above_gap || player_blow_gap)
    }
}

fn main() -> BError {
    // 创建一个 80x50 的游戏上下文并设置标题
    let context = BTermBuilder::simple80x50()
        .with_title("Flappy Dragon")
        .build()?;

    // 启动游戏循环
    main_loop(context, State::new())
}
