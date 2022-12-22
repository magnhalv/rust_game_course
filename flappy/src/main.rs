use std::path::Path;

use bracket_lib::prelude::*;

const SCREEN_WIDTH: i32 = 160;
const SCREEN_HEIGHT: i32 = 128;
const FRAME_DURATION: f32 = 5.0;

const RENDER_OFFSET_X : i32 = 5;

const TEXT_ID: usize = 0;
const SPRITE_ID: usize = 1;

const SPRITE_SIZE: i32 = 4;

const NUM_ANIMATION_FRAMES: usize = 4;
const DEFAULT_ANIMATION_FRAME: usize = 1;
const ANIMATION_FRAME_LENGTH: f32 = 20.0;

struct State {
    mode: GameMode,
    player: Player,
    obstacle: Obstacle,
    score: i32
}

struct Player {
    x: i32,
    y: i32,
    velocity: f32,
    curr_animation_index: usize,
    curr_frame_time: f32,
    is_animating: bool
}

struct Obstacle {
    x: i32,
    gap_y: i32,
    size: i32
}

struct BBox {
    x: i32,
    y: i32,
    x_end: i32,
    y_end: i32
}

impl BBox {
    
    fn is_hit(self: &BBox, b: &BBox) -> bool {
        let x_start_inside = self.x >= b.x && self.x < b.x_end;
        let x_end_inside = self.x_end >= b.x && self.x_end < b.x_end;

        let x_inside = x_start_inside || x_end_inside;

        let y_start_inside = self.y >= b.y && self.y < b.y_end;
        let y_end_inside = self.y_end >= b.y && self.y_end < b.y_end;

        let y_inside = y_start_inside || y_end_inside;

        x_inside && y_inside
    }
}

fn screen_x(world_x: i32) -> i32 {
    return world_x*SPRITE_SIZE
}

fn screen_y(world_y: i32) -> i32 {
    return world_y*SPRITE_SIZE
}

impl Obstacle {
    fn new(x: i32, score: i32) -> Self {
        let mut random = RandomNumberGenerator::new();
        Obstacle { x, gap_y: random.range(SPRITE_SIZE*4, SCREEN_HEIGHT-(SPRITE_SIZE*4)), size: i32::max(2, 32-score) }
    }

    fn render(&mut self, ctx: &mut BTerm, player_x: i32) {
        let screen_x = self.x - player_x + RENDER_OFFSET_X;
        let half_size = self.size/2;

        let end_upper = (self.gap_y - half_size)/SPRITE_SIZE;

        for y in 0..end_upper {            
            ctx.add_sprite(Rect::with_size(screen_x, y*SPRITE_SIZE, SPRITE_SIZE, SPRITE_SIZE),  0, RGBA::from_f32(1.0, 1.0, 1.0, 1.0), 4);        
        }

        let num_bricks_lower  = (SCREEN_HEIGHT - (self.gap_y + half_size))/4 + 1;
        for y in 0..num_bricks_lower {            
            ctx.add_sprite(Rect::with_size(screen_x, self.gap_y + half_size + y*SPRITE_SIZE, SPRITE_SIZE, SPRITE_SIZE),  0, RGBA::from_f32(1.0, 1.0, 1.0, 1.0), 4);        
        }        
    }

    fn hit_obstacle(&self, player: &Player) -> bool {
        let half_size = self.size / 2;

        let player_bbox = BBox { x: player.x, y: player.y, x_end: player.x + SPRITE_SIZE, y_end: player.y + SPRITE_SIZE};        
        let upper_bbox = BBox { x: self.x, y: 0, x_end: self.x + SPRITE_SIZE, y_end: self.gap_y - half_size};
        let lower_bbox = BBox { x: self.x, y: self.gap_y + half_size, x_end: self.x + SPRITE_SIZE, y_end: SCREEN_HEIGHT};

        
        (player_bbox.is_hit(&upper_bbox) || player_bbox.is_hit(&lower_bbox))
    }
    

}

impl Player {
    fn new(x: i32, y: i32) -> Self {
        Player { x, y, velocity: 0.0, curr_animation_index: DEFAULT_ANIMATION_FRAME, curr_frame_time: 0.0, is_animating: false }
    }

    fn render(&mut self, ctx: &mut BTerm) {        
        ctx.add_sprite(Rect::with_size(RENDER_OFFSET_X, self.y, SPRITE_SIZE, SPRITE_SIZE),  0, RGBA::from_f32(1.0, 1.0, 1.0, 1.0), self.curr_animation_index);        
    }

    fn gravity_and_move(&mut self, dt: f32) {
        if self.velocity < 8.0 {
            self.velocity += 0.4 * dt;
        }

        let mut vel = (self.velocity * dt) as i32;
        
        if vel == 0 {
            if self.velocity > 0.0 {
                vel = 1;
            }
        }
        self.y += vel;
        self.x += (6.0 * dt) as i32;

        if self.y < 0 {
            self.y = 0;
        }
    }

    fn update_animation(&mut self, ctx: &BTerm) {
        if !self.is_animating {
            return;
        }

        self.curr_frame_time += ctx.frame_time_ms;

        if self.curr_frame_time > ANIMATION_FRAME_LENGTH {
            self.curr_frame_time = 0.0;
            self.curr_animation_index += 1;

            if self.curr_animation_index >= NUM_ANIMATION_FRAMES {
                self.curr_animation_index = 0;
            }

            if self.curr_animation_index == DEFAULT_ANIMATION_FRAME {
                self.is_animating = false;
            }
        }
    }

    fn flap(&mut self) {
        self.velocity = -4.0;
    }
}

impl State {
    fn new() -> Self {
        State {
            mode: GameMode::Menu,
            player: Player::new(5, 25),            
            obstacle: Obstacle::new(SCREEN_WIDTH, 0),
            score: 0
        }
    }

    fn play(&mut self, ctx: &mut BTerm) {
        ctx.set_active_console(TEXT_ID);
        ctx.cls_bg(NAVY);                
    
        let dt: f32 = ctx.frame_time_ms/100.0;
        self.player.gravity_and_move(dt);            
   
        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.player.flap();

            if !self.player.is_animating {
                self.player.is_animating = true;
            }
        }

        self.player.update_animation(ctx);

        ctx.print(0, 0, "Press SPACE to flap.");
        ctx.print(0, 1, &format!("Score: {}", self.score));
        


        ctx.set_active_console(SPRITE_ID);
        ctx.cls();

        self.player.render(ctx);        
        self.obstacle.render(ctx, self.player.x);
        if self.player.x > (self.obstacle.x + SPRITE_SIZE + RENDER_OFFSET_X) {
            self.score += 1;
            self.obstacle = Obstacle::new(self.player.x + SCREEN_WIDTH, self.score);
        }        

        if self.player.y > SCREEN_HEIGHT || self.obstacle.hit_obstacle(&self.player) {
            self.mode = GameMode::End;
        }        
    }

    fn restart(&mut self) {
        self.player = Player::new(5, 25);        
        self.mode = GameMode::Playing;
        self.obstacle = Obstacle::new(SCREEN_WIDTH, 0);
        self.score = 0;
    }

    fn main_menu(&mut self, ctx: &mut BTerm) {
        ctx.set_active_console(TEXT_ID);
        ctx.cls();
        ctx.print_centered(5, "Welcome to Flappy Dragon");
        ctx.print_centered(8, "(P) Play Game");
        ctx.print_centered(9, "(Q) Quit Game");

        //ctx.set_active_console(1);
        //ctx.add_sprite(Rect::with_size(10, 10, 1, 1), 0, RGBA::from_f32(1.0, 1.0, 1.0, 1.0), 0);

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn dead(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.set_active_console(TEXT_ID);
        ctx.print_centered(5, "You are dead :(");
        ctx.print_centered(6, &format!("You earned {} points", self.score));
        ctx.print_centered(8, "(P) Play Again");
        ctx.print_centered(9, "(Q) Quit Game");

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
    fn tick(&mut self, ctx: &mut BTerm) {
        if let Some(VirtualKeyCode::Escape) = ctx.key {
            ctx.quitting = true;
        }
            
        match self.mode {
            GameMode::Menu => self.main_menu(ctx),
            GameMode::Playing => self.play(ctx),
            GameMode::End => self.dead(ctx),
        }
    }
}

enum GameMode {
    Menu, Playing, End
}

fn main() -> BError {
    let path = Path::new("assets").join("spritesheet.png");
    let sprite_sheet = SpriteSheet::new(path.to_str().unwrap())
    .add_sprite(Rect::with_size(0, 0, 32, 32))
    .add_sprite(Rect::with_size(32, 0, 32, 32))
    .add_sprite(Rect::with_size(64, 0, 32, 32))
    .add_sprite(Rect::with_size(96, 0, 32, 32))
    .add_sprite(Rect::with_size(128, 0, 32, 32));

    let context = BTermBuilder::simple(SCREEN_WIDTH, SCREEN_HEIGHT)       
        .unwrap()
        .with_sprite_console(SCREEN_WIDTH, SCREEN_HEIGHT, 0)
        .with_title("Flappy Dragon")
        .with_sprite_sheet(sprite_sheet)
        .build()?;
    
    main_loop(context, State::new())

}
