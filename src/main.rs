use ggez::{graphics::*, input::keyboard::KeyCode, *};
use mint::Point2;
use std::collections::VecDeque;

use touhoulang::*;
use touhoulang_macro::Evaluate;

type Story = Vec<StoryLine>;
type UIMenu = VecDeque<UISelectable<Text>>;

// ------------------------------------------
// Entities
// ------------------------------------------

struct Player {
    health: Health,
    body: Body,
    spell: Spell,
}

struct Enemy {
    health: Health,
    body: Body,
    spell: Spell,

    move_timer: Timer,
    directions: Vec<f32>,
}

#[derive(Clone)]
struct Body {
    sprite: Sprite,
    position: Point2<f32>,
    direction: Point2<f32>,
    speed: f32,
}

struct Health {
    health: u32,
    max_health: u32,
    on_hit: Option<fn(health: u32)>,
}

struct Spell {
    bullets: Vec<Bullet>,
    shot_timer: Timer,
}

#[derive(Clone)]
struct Bullet {
    body: Body,
    is_visible: bool,
}

#[derive(Clone)]
struct Sprite {
    image: Image,
    color: Color,
}

struct Timer {
    time: std::time::Duration,
    delay: f32,
}

// ------------------------------------------
// UI
// ------------------------------------------

struct Particle {
    bullet: Bullet,
    timer: Timer,
}

#[derive(PartialEq)]
enum GameState {
    Combat,
    Paused,
    Cinematic,
}

struct StoryLine {
    text: Text,
    sprite: Sprite,
    pos: Point2<f32>,
    color: Color,
}

struct UISelectable<T: Drawable> {
    img: T,
    pos: Point2<f32>,
    color: Color,
    select_color: Color,
    action: fn(&mut Context, &mut State),
}

struct Screen {
    width: f32,
    height: f32,
}

struct State {
    uis: VecDeque<UIMenu>,
    last_update: std::time::SystemTime,

    gamestate: GameState,
    story: Story,

    screen: Screen,
    background: Image,

    player: Option<Player>,
    enemy: Option<Enemy>,

    texts: Vec<Text>,
    particles: Vec<Particle>,
}

// ------------------------------------------
// Serialization
// ------------------------------------------

#[derive(Evaluate, Default)]
struct Globals {
    background: String,
    player: InitObject,
    enemy: InitObject,
}

#[derive(Evaluate, Default)]
struct InitData {
    amount: usize,
    health: u32,
    speed: f32,
}

#[derive(Evaluate, Default)]
struct InitObject {
    data: InitData,
    bullet: InitData,
}

trait Distance {
    fn distance(&self, other: &Self) -> f32;
}

impl InitObject {
    fn health(&self) -> u32 {
        self.data.health
    }

    fn speed(&self) -> f32 {
        self.data.speed
    }
}

impl Particle {
    fn new(sprite: &Sprite, ttl: f32, position: [f32; 2], direction: [f32; 2], speed: f32) -> Self {
        let mut bullet = Bullet::new(sprite, direction, speed);
        bullet.body.position = Point2::from(position);
        bullet.is_visible = true;

        Self {
            bullet,
            timer: Timer::new(ttl),
        }
    }

    fn update(&mut self, ctx: &Context) {
        if self.timer.ready(ctx) {
            self.bullet.is_visible = false;
        } else {
            self.bullet.update();
        }
    }
}

impl Health {
    fn take_damage(&mut self, damage: u32) {
        self.health = self.health.saturating_sub(damage);
        if let Some(on_hit) = self.on_hit {
            on_hit(self.health);
        }
    }

    fn is_alive(&self) -> bool {
        self.health > 0
    }

    fn percentage(&self) -> f32 {
        self.health as f32 / self.max_health as f32
    }
}

impl Spell {
    fn new(bullet: Bullet, bullets_size: usize, delay: f32) -> Self {
        Self {
            bullets: std::iter::repeat(bullet).take(bullets_size).collect(),
            shot_timer: Timer::new(delay),
        }
    }

    fn spawn(&mut self, ctx: &Context, position: &Point2<f32>) {
        if self.shot_timer.ready(ctx) {
            self.bullets
                .iter_mut()
                .find(|x| !x.is_visible)
                .map(|bullet| {
                    bullet.body.position = *position;
                    bullet.is_visible = true;
                });
        }
    }

    fn for_each_visible(&self, f: impl FnMut(&Bullet)) {
        self.bullets.iter().filter(|x| x.is_visible).for_each(f);
    }

    fn for_each_visible_mut(&mut self, f: impl FnMut(&mut Bullet)) {
        self.bullets.iter_mut().filter(|x| x.is_visible).for_each(f);
    }
}

impl Timer {
    fn new(delay: f32) -> Self {
        Self {
            time: std::time::Duration::new(0, 0),
            delay,
        }
    }

    fn ready(&mut self, ctx: &Context) -> bool {
        self.time += ctx.time.delta();
        let ready = self.time.as_secs_f32() > self.delay;
        if ready {
            self.time = std::time::Duration::new(0, 0);
        }
        ready
    }
}

impl Body {
    fn new(sprite: &Sprite, position: [f32; 2], direction: [f32; 2], speed: f32) -> Self {
        Self {
            position: Point2::from(position),
            direction: Point2::from(direction),
            speed,
            sprite: sprite.clone(),
        }
    }
}

impl Bullet {
    fn new(sprite: &Sprite, direction: [f32; 2], speed: f32) -> Self {
        Self {
            body: Body::new(&sprite, [0.0, 0.0], direction, speed),
            is_visible: false,
        }
    }

    fn update(&mut self) {
        let Point2 { x: dx, y: dy } = self.body.direction;
        let speed = self.body.speed;

        self.body.position.x += dx * speed;
        self.body.position.y += dy * speed;
    }

    fn collided(&self, _other: &Point2<f32>, hitbox_size: f32) -> bool {
        self.body.position.distance(_other) < hitbox_size
    }
}

impl Player {
    fn new(sprite: &Sprite, health: u32, bullet: Bullet, bullets_size: usize) -> Self {
        Self {
            health: Health {
                health,
                max_health: health,
                on_hit: None,
            },
            body: Body::new(sprite, [350.0, 350.0], [0.0, 0.0], 5.0),
            spell: Spell::new(bullet, bullets_size, 0.1),
        }
    }

    fn update(&mut self, ctx: &Context, enemy: &mut Option<Enemy>) {
        self.spell.for_each_visible_mut(|bullet| {
            bullet.update();

            if let Some(enemy) = enemy {
                if bullet.collided(&enemy.body.position, 100.) {
                    enemy.health.take_damage(1);
                    bullet.is_visible = false;
                }
            }

            if bullet.body.position.x < 0.0 || bullet.body.position.y < 0.0 {
                bullet.is_visible = false;
            }
        });
        self.spell.spawn(&ctx, &self.body.position);
    }
}

impl Enemy {
    fn new(sprite: &Sprite, health: u32, speed: f32, bullet: Bullet, bullets_size: usize) -> Self {
        Self {
            health: Health {
                health,
                max_health: health,
                on_hit: Some(|hp| println!("Enemy Health: {hp}")),
            },
            body: Body::new(sprite, [350.0, 100.0], [1.0, 0.0], speed),
            spell: Spell::new(bullet, bullets_size, 0.5),
            directions: vec![-1., 0., 1., 0., 1., 0., -1., 0.],
            move_timer: Timer::new(1.5),
        }
    }

    fn update(&mut self, ctx: &Context, player: &mut Option<Player>, screen: &Screen) {
        self.move_auto(&ctx);
        self.spell.for_each_visible_mut(|bullet| {
            bullet.update();

            if let Some(player) = player {
                if bullet.collided(&player.body.position, 25.) {
                    player.health.take_damage(1);
                    bullet.is_visible = false;
                }
            }

            if bullet.body.position.x < 0.0 || bullet.body.position.y > screen.height {
                bullet.is_visible = false;
            }
        });
        self.spell.spawn(&ctx, &self.body.position);
    }

    fn move_auto(&mut self, ctx: &Context) {
        if self.move_timer.ready(ctx) {
            self.directions.rotate_left(1);
        }

        let vel = self.directions.first().unwrap_or(&0.0);

        self.body.position = Point2 {
            x: self.body.position.x + vel * self.body.speed,
            y: self.body.position.y,
        }
    }
}

impl StoryLine {
    fn new(text: &str, sprite: Sprite, pos: [f32; 2], color: Color) -> Self {
        Self {
            text: centered_text(text),
            sprite,
            pos: pos.into(),
            color,
        }
    }
}

impl Distance for Point2<f32> {
    fn distance(&self, other: &Self) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

macro_rules! story {
    ($($spr:ident: $text:expr, $pos:tt,)*) => {{
        let mut story = vec![$(StoryLine::new($text, $spr, $pos, Color::WHITE)),*];
        story.reverse();
        story
    }};
    ($($spr:ident: $text:expr, $pos:tt, $color:expr,)*) => {{
        let mut story = vec![$(StoryLine::new($text, $spr, $pos, $color)),*];
        story.reverse();
        story
    }}
}

macro_rules! rect {
    ($ctx:ident, $w:expr, $h:expr, ($r:literal, $g:literal, $b:literal, $a:literal)) => {
        Mesh::new_rectangle(
            $ctx,
            DrawMode::fill(),
            Rect::new(0.0, 0.0, $w, $h),
            Color::from_rgba($r, $g, $b, $a),
        )
        .unwrap()
    };
}

macro_rules! draw_at {
    ($canvas:ident, $ref:expr, ($x:expr, $y:expr)) => {
        $canvas.draw($ref, DrawParam::default().dest([$x, $y]))
    };
    ($canvas:ident, $ref:expr, ($x:expr, $y:expr), $color: expr) => {
        $canvas.draw($ref, DrawParam::default().dest([$x, $y]).color($color))
    };
}

fn centered_text(text: &str) -> Text {
    Text::new(TextFragment {
        text: text.to_owned(),
        scale: Some(PxScale::from(40.0)),
        ..Default::default()
    })
    .set_layout(TextLayout {
        h_align: TextAlign::Middle,
        v_align: TextAlign::Middle,
    })
    .to_owned()
}

fn pause_menu() -> UIMenu {
    [
        UISelectable {
            img: centered_text("Resume"),
            pos: Point2 { x: 0., y: -100. },
            color: Color::WHITE,
            select_color: Color::YELLOW,
            action: |_, state| {
                state.uis.remove(0);
                state.gamestate = if state.story.is_empty() {
                    GameState::Combat
                } else {
                    GameState::Cinematic
                };
            },
        },
        UISelectable {
            img: centered_text("Reset"),
            pos: Point2 { x: 0., y: 0. },
            color: Color::WHITE,
            select_color: Color::YELLOW,
            action: |ctx, state| state.restart(ctx),
        },
        UISelectable {
            img: centered_text("Quit"),
            pos: Point2 { x: 0., y: 100. },
            color: Color::WHITE,
            select_color: Color::RED,
            action: |ctx, _| ctx.request_quit(),
        },
    ]
    .into()
}

fn get_script_mod_date() -> std::time::SystemTime {
    let metadata = std::fs::metadata("script.th").unwrap();
    metadata.modified().unwrap()
}

impl State {
    fn new(ctx: &Context) -> Self {
        let script_text = std::fs::read_to_string("script.th").unwrap();
        let init_panic = std::panic::catch_unwind(|| Globals::from_str(&script_text));

        let init = if let Ok(ref init) = init_panic {
            init
        } else {
            &Globals::default()
        };

        let b_spr = Sprite {
            image: load_image(ctx, BULLET_IMG_PATH),
            color: Color::WHITE,
        };

        let p_spr = Sprite {
            image: load_image(ctx, PLAYER_IMG_PATH),
            color: Color::WHITE,
        };

        let player = Player::new(
            &p_spr,
            init.player.health(),
            Bullet::new(&b_spr, DIR_UP, init.player.bullet.speed),
            init.player.bullet.amount,
        );

        let e_spr = Sprite {
            image: load_image(ctx, ENEMY_IMG_PATH),
            color: Color::BLACK,
        };

        let enemy = Enemy::new(
            &p_spr,
            init.enemy.health(),
            init.enemy.speed(),
            Bullet::new(&b_spr, DIR_DOWN, init.enemy.bullet.speed),
            init.enemy.bullet.amount,
        );

        let (width, height) = ctx.gfx.size();
        let screen = Screen { width, height };

        let background = load_image(ctx, format!("/{}/", init.background).as_str());

        let story = if let Err(e) = init_panic {
            let msg = e.downcast_ref::<String>().unwrap();
            story! {
                p_spr: msg, [0., 0.], Color::BLACK,
            }
        } else {
            story! {
                p_spr: "The story begins...", [0., 0.],
                e_spr: "I'm going to kill you!", [-width * 0.7, 0.],
            }
        };

        Self {
            gamestate: GameState::Cinematic,
            last_update: get_script_mod_date(),

            screen,
            story,

            uis: VecDeque::new(),
            player: Some(player),
            enemy: Some(enemy),
            background,

            particles: vec![],
            texts: vec![],
        }
    }

    fn draw_body(&self, canvas: &mut Canvas, body: &Body, size: f32, color: Color) {
        canvas.draw(
            &body.sprite.image,
            DrawParam::new()
                .dest(body.position)
                .scale([size, size])
                .color(color)
                .offset([0.5, 0.5]),
        );
    }

    fn on_combat_update(&mut self, ctx: &mut Context) -> GameResult {
        for key in [KeyCode::W, KeyCode::S, KeyCode::A, KeyCode::D] {
            if ctx.keyboard.is_key_pressed(key) {
                let dir = match key {
                    KeyCode::W => DIR_UP,
                    KeyCode::S => DIR_DOWN,
                    KeyCode::A => DIR_LEFT,
                    KeyCode::D => DIR_RIGHT,
                    _ => return Ok(()),
                };

                if let Some(Player { ref mut body, .. }) = self.player {
                    body.position.x += dir[0] * body.speed;
                    body.position.y += dir[1] * body.speed;
                }
            }
        }

        if let Some(ref mut player) = self.player {
            player.update(&ctx, &mut self.enemy);

            if !player.health.is_alive() {
                let Point2 { x, y } = player.body.position;
                let sprite = &player.spell.bullets.first().unwrap().body.sprite;

                for dir in [DIR_UP, DIR_DOWN, DIR_LEFT, DIR_RIGHT] {
                    self.particles
                        .push(Particle::new(sprite, 2.0, [x, y], dir, 5.0));
                }

                self.texts
                    .push(centered_text("You died! Press R to restart."));

                self.player = None;
            }
        }

        if let Some(ref mut enemy) = self.enemy {
            enemy.update(&ctx, &mut self.player, &self.screen);

            if !enemy.health.is_alive() {
                let Point2 { x, y } = enemy.body.position;
                let sprite = &enemy.spell.bullets.first().unwrap().body.sprite;

                for dir in [DIR_UP, DIR_DOWN, DIR_LEFT, DIR_RIGHT] {
                    self.particles
                        .push(Particle::new(sprite, 2.0, [x, y], dir, 5.0));
                }

                self.texts
                    .push(centered_text("You win! Press R to restart."));

                self.enemy = None;
            }
        }

        self.particles.retain_mut(|particle| {
            particle.update(ctx);
            particle.bullet.is_visible
        });

        Ok(())
    }

    fn restart(&mut self, ctx: &mut Context) {
        println!("Game Restarted!");
        *self = Self::new(ctx);
    }
}

const PLAYER_IMG_PATH: &str = "/sakuya.png";
const ENEMY_IMG_PATH: &str = "/sakuya.png";
const BULLET_IMG_PATH: &str = "/isaac.png";

const DIR_UP: [f32; 2] = [0.0, -1.0];
const DIR_DOWN: [f32; 2] = [0.0, 1.0];
const DIR_LEFT: [f32; 2] = [-1.0, 0.0];
const DIR_RIGHT: [f32; 2] = [1.0, 0.0];

impl ggez::event::EventHandler<GameError> for State {
    fn resize_event(&mut self, _ctx: &mut Context, w: f32, h: f32) -> Result<(), GameError> {
        self.screen.width = w;
        self.screen.height = h;
        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        input: input::keyboard::KeyInput,
        _repeated: bool,
    ) -> Result<(), GameError> {
        match input.keycode {
            Some(KeyCode::Return) | Some(KeyCode::Space) if !_repeated => match self.gamestate {
                GameState::Cinematic => {
                    if self.story.pop().is_none() || self.story.is_empty() {
                        self.gamestate = GameState::Combat;
                    }
                }
                GameState::Paused => {
                    if let Some(elem) = self.uis[0].front() {
                        (elem.action)(ctx, self);
                    }
                }
                _ => {}
            },
            Some(KeyCode::Escape) if !_repeated && self.gamestate != GameState::Paused => {
                self.gamestate = GameState::Paused;
                self.uis.push_front(pause_menu());
            }
            Some(KeyCode::R) if !_repeated => {
                self.restart(ctx);
            }
            Some(key) if self.gamestate == GameState::Paused => match key {
                KeyCode::Down | KeyCode::Right | KeyCode::S | KeyCode::D => {
                    if let Some(elem) = self.uis[0].pop_front() {
                        self.uis[0].push_back(elem);
                    }
                }
                KeyCode::Up | KeyCode::Left | KeyCode::W | KeyCode::A => {
                    if let Some(elem) = self.uis[0].pop_back() {
                        self.uis[0].push_front(elem);
                    }
                }
                _ => {}
            },
            _ => {}
        }

        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let curr = get_script_mod_date();
        if curr != self.last_update {
            self.last_update = curr;
            self.restart(ctx);
        }

        match self.gamestate {
            GameState::Combat => self.on_combat_update(ctx),
            _ => Ok(()),
        }
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = Canvas::from_frame(ctx, Color::from_rgb(0x2b, 0x2c, 0x2f));

        let width = self.screen.width;
        let height = self.screen.height;

        let half_width = width * 0.5;
        let half_height = height * 0.5;

        let (w, h) = (
            self.background.width() as f32,
            self.background.height() as f32,
        );

        canvas.draw(
            &self.background,
            DrawParam::default().scale([width / w, height / h]),
        );

        if let Some(ref enemy) = self.enemy {
            self.draw_body(&mut canvas, &enemy.body, 0.2, Color::BLACK);
            enemy.spell.for_each_visible(|bullet| {
                self.draw_body(&mut canvas, &bullet.body, 0.05, Color::RED);
            });

            let healthbar = rect!(
                ctx,
                enemy.health.percentage() * 100.,
                10.0,
                (255, 0, 0, 127)
            );

            draw_at!(
                canvas,
                &healthbar,
                (enemy.body.position.x - 50.0, enemy.body.position.y - 90.0)
            );
        }

        if let Some(ref player) = self.player {
            self.draw_body(&mut canvas, &player.body, 0.12, Color::WHITE);
            player.spell.for_each_visible(|bullet| {
                self.draw_body(&mut canvas, &bullet.body, 0.05, Color::CYAN);
            });
        }

        self.particles.iter().for_each(|particle| {
            self.draw_body(&mut canvas, &particle.bullet.body, 0.05, Color::MAGENTA);
        });

        self.texts
            .iter()
            .for_each(|text| draw_at!(canvas, text, (half_width, half_height)));

        if let Some(line) = self.story.last() {
            draw_at!(canvas, &line.text, (half_width, half_height), line.color);
            draw_at!(
                canvas,
                &line.sprite.image,
                (width * 0.5 + line.pos.x, height * 0.5 + line.pos.y),
                line.sprite.color
            );
        }

        // TODO: limited pauses, with breaking effect after unpausing
        if self.gamestate == GameState::Paused {
            let background = rect!(ctx, width, height, (0, 0, 0, 127));
            draw_at!(canvas, &background, (0.0, 0.0));
        }

        self.uis.iter().for_each(|ui| {
            if let Some(elem) = ui.front() {
                let Point2 { x, y } = elem.pos;
                draw_at!(
                    canvas,
                    &elem.img,
                    (half_width + x, half_height + y),
                    elem.select_color
                );
            }

            ui.iter().skip(1).for_each(|elem| {
                let Point2 { x, y } = elem.pos;
                draw_at!(
                    canvas,
                    &elem.img,
                    (half_width + x, half_height + y),
                    elem.color
                );
            });
        });

        canvas.finish(ctx)
    }
}

fn load_image(ctx: &Context, path: &str) -> Image {
    Image::from_path(ctx, path).unwrap_or(Image::from_color(ctx, 1, 1, None))
}

fn main() -> GameResult {
    let (ctx, event_loop) = ContextBuilder::new("Touhou Engine", "Rontero")
        .add_resource_path(std::path::PathBuf::from("./assets"))
        .default_conf(conf::Conf::new())
        .build()?;

    let state = State::new(&ctx);
    event::run(ctx, event_loop, state);
}
