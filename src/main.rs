mod physics;

use ggez::{graphics::*, input::keyboard::KeyCode, *};
use mint::Point2;
use physics::{Movable, Rigidbody};
use std::collections::VecDeque;

use touhoulang::*;
use touhoulang_macro::Evaluate;

#[derive(Clone)]
struct Body {
    rigidbody: Rigidbody,
    sprite: Image,
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

struct Enemy {
    health: Health,
    body: Body,
    spell: Spell,

    move_timer: Timer,
    directions: Vec<f32>,
}

struct Player {
    health: Health,
    body: Body,
    spell: Spell,
}

struct Sprite {
    image: Image,
    color: Color,
}

struct Timer {
    time: std::time::Duration,
    delay: f32,
}

struct StoryLine {
    text: Text,
    sprite: Sprite,
    pos: Point2<f32>,
}

struct Particle {
    bullet: Bullet,
    timer: Timer,
}

struct Health {
    health: u32,
    max_health: u32,
    on_hit: fn(health: u32),
}

#[derive(PartialEq)]
enum GameState {
    Combat,
    Paused,
    Cinematic,
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
    pause_menu: VecDeque<UISelectable<Text>>,
    gamestate: GameState,
    story: Story,

    screen: Screen,
    background: Option<Image>,

    texts: Vec<Text>,
    players: Vec<Player>,
    enemies: Vec<Enemy>,
    particles: Vec<Particle>,
}

type Story = Vec<StoryLine>;

impl Particle {
    fn new(ctx: &Context, ttl: f32, position: [f32; 2], velocity: [f32; 2]) -> Self {
        let mut bullet = Bullet::new(ctx, velocity);
        bullet.body.set_position(position[0], position[1]);
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
            self.bullet.body.move_by(*self.bullet.body.velocity());
        }
    }
}

impl Health {
    fn take_damage(&mut self, damage: u32) {
        self.health = self.health.saturating_sub(damage);
        (self.on_hit)(self.health);
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
                    bullet.body.set_position(position.x, position.y);
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
    fn new(velocity: [f32; 2], position: [f32; 2], ctx: &Context, image_path: &str) -> Self {
        Self {
            rigidbody: Rigidbody {
                position: Point2::from(position),
                velocity: velocity.into(),
            },
            sprite: load_image(ctx, image_path),
        }
    }
}

impl Bullet {
    fn new(ctx: &Context, velocity: [f32; 2]) -> Self {
        Self {
            body: Body::new(velocity, [0.0, 0.0], ctx, BULLET_IMG_PATH),
            is_visible: false,
        }
    }

    fn update(&mut self) {
        self.body.move_by(*self.body.velocity());
    }

    fn collided(&self, _other: &Point2<f32>, hitbox_size: f32) -> bool {
        self.body.position().distance(_other) < hitbox_size
    }
}

impl Player {
    fn new(ctx: &Context, health: u32, bullet: Bullet, bullets_size: usize) -> Self {
        Self {
            health: Health {
                health,
                max_health: health,
                on_hit: |_| {},
            },
            body: Body::new([5.0, 5.0], [350.0, 350.0], ctx, PLAYER_IMG_PATH),
            spell: Spell::new(bullet, bullets_size, 0.1),
        }
    }
}

impl Enemy {
    fn new(ctx: &Context, health: u32, speed: f32, bullet: Bullet, bullets_size: usize) -> Self {
        Self {
            health: Health {
                health,
                max_health: health,
                on_hit: |hp| println!("Enemy Health: {hp}"),
            },
            body: Body::new([speed, 0.0], [350.0, 100.0], ctx, ENEMY_IMG_PATH),
            spell: Spell::new(bullet, bullets_size, 0.5),
            directions: vec![-1., 0., 1., 0., 1., 0., -1., 0.],
            move_timer: Timer::new(1.5),
        }
    }

    fn move_auto(&mut self, ctx: &Context) {
        if self.move_timer.ready(ctx) {
            self.directions.rotate_left(1);
        }

        let vel = self.directions.first().unwrap_or(&0.0);
        let speed = self.body.speed();

        self.body.move_by([speed * vel, 0.0].into());
    }
}

impl StoryLine {
    fn new(text: &str, sprite: Sprite, pos: [f32; 2]) -> Self {
        Self {
            text: centered_text(text),
            sprite,
            pos: pos.into(),
        }
    }
}

trait Distance {
    fn distance(&self, other: &Self) -> f32;
}

impl Distance for Point2<f32> {
    fn distance(&self, other: &Self) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

impl Movable for Body {
    fn get_rigidbody(&self) -> &Rigidbody {
        &self.rigidbody
    }

    fn set_position(&mut self, x: f32, y: f32) {
        self.rigidbody.position.x = x;
        self.rigidbody.position.y = y;
    }
}

impl Movable for Player {
    fn get_rigidbody(&self) -> &Rigidbody {
        &self.body.rigidbody
    }

    fn set_position(&mut self, x: f32, y: f32) {
        self.body.set_position(x, y);
    }
}

impl Movable for Enemy {
    fn get_rigidbody(&self) -> &Rigidbody {
        &self.body.rigidbody
    }

    fn set_position(&mut self, x: f32, y: f32) {
        self.body.set_position(x, y);
    }
}

#[derive(Evaluate, Default)]
struct Globals {
    background: String,
}

#[derive(Evaluate, Default)]
struct Bull {
    amount: usize,
    speed: f32,
}

#[derive(Evaluate, Default)]
struct Sakuya {
    bullet: Bull,
    health: u32,
}

#[derive(Evaluate, Default)]
struct Reimu {
    bullet: Bull,
    health: u32,
    speed: f32,
}

macro_rules! check_game_finish {
    ($self:ident, $ctx: ident, $targets:ident, $death:ident, $text:literal) => {
        if $death {
            $self.$targets.retain(|t| {
                let is_alive = t.health.is_alive();
                if !is_alive {
                    let (x, y) = (t.x(), t.y());
                    for dir in [DIR_UP, DIR_DOWN, DIR_LEFT, DIR_RIGHT] {
                        $self.particles.push(Particle::new(
                            $ctx,
                            2.0,
                            [x, y],
                            dir.map(|x| x * 5.0),
                        ));
                    }
                }
                is_alive
            });

            if $self.$targets.is_empty() {
                $self.texts.push(centered_text($text));
            }
        }
    };
}

macro_rules! bullet_hit {
    ($targets:expr, $bullet:ident, $hitbox:literal) => {
        $targets.iter_mut().for_each(|t| {
            if $bullet.collided(t.position(), $hitbox) {
                t.health.take_damage(1);
                $bullet.is_visible = false;
            }
        });
    };
}

macro_rules! bullets_hit {
    ($shooter:expr, $targets:expr, $hitbox:literal, $ylimit: expr) => {
        $shooter.spell.for_each_visible_mut(|bullet| {
            bullet.update();
            bullet_hit!($targets, bullet, $hitbox);
            if bullet.body.y() < 0.0 || bullet.body.y() > $ylimit {
                bullet.is_visible = false;
            }
        });
    };
}

impl State {
    fn new(ctx: &Context) -> Self {
        let script_text = std::fs::read_to_string("script.th").unwrap();
        let values = parser::parse(tokenizer::tokenize(&script_text));
        let bullet = |velocity: [f32; 2]| Bullet::new(ctx, velocity.into());

        let mut players = vec![];
        if let Some(_) = values.get("sakuya") {
            let mut sakuya = Sakuya {
                health: 1,
                bullet: Bull {
                    amount: 8,
                    speed: 2.0,
                },
            };
            sakuya.evaluate(values.clone());

            players.push(Player::new(
                ctx,
                sakuya.health,
                bullet(DIR_UP.map(|x| x * sakuya.bullet.speed)),
                sakuya.bullet.amount,
            ));
        }

        let mut enemies = vec![];
        if let Some(_) = values.get("reimu") {
            let mut reimu = Reimu {
                health: 200,
                speed: 2.0,
                bullet: Bull {
                    amount: 5,
                    speed: 5.0,
                },
            };
            reimu.evaluate(values.clone());

            enemies.push(Enemy::new(
                ctx,
                reimu.health,
                reimu.speed,
                bullet(DIR_DOWN.map(|x| x * reimu.bullet.speed)),
                reimu.bullet.amount,
            ));
        }

        let mut globals = Globals::default();
        globals.evaluate(values.clone());

        let background = values
            .get("background")
            .map(|_| load_image(ctx, format!("/{}/", globals.background).as_str()));

        let (win_w, win_h) = ctx.gfx.size();

        let screen = Screen {
            width: win_w,
            height: win_h,
        };

        let player_spr = Sprite {
            image: load_image(ctx, PLAYER_IMG_PATH),
            color: Color::WHITE,
        };

        let enemy_spr = Sprite {
            image: load_image(ctx, ENEMY_IMG_PATH),
            color: Color::BLACK,
        };

        let story = vec![
            StoryLine::new("I'm going to kill you!", enemy_spr, [-win_w * 0.7, 0.]),
            StoryLine::new("The story begins...", player_spr, [0., 0.]),
        ];

        Self {
            gamestate: GameState::Cinematic,
            screen,
            story,

            pause_menu: vec![
                UISelectable {
                    img: centered_text("Resume"),
                    pos: Point2 { x: 0., y: -100. },
                    color: Color::WHITE,
                    select_color: Color::YELLOW,
                    action: |_, state| {
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
            .into(),

            players,
            enemies,
            background,

            particles: vec![],
            texts: vec![],
        }
    }

    fn draw_body(&self, canvas: &mut Canvas, body: &Body, size: f32, color: Color) {
        canvas.draw(
            &body.sprite,
            DrawParam::new()
                .dest(*body.position())
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

                self.players
                    .iter_mut()
                    .for_each(|player| player.move_by(dir.map(|x| x * player.speed()).into()));
            }
        }

        self.enemies.iter_mut().for_each(|enemy| {
            enemy.move_auto(&ctx);
            bullets_hit!(enemy, self.players, 25., self.screen.height);
            enemy.spell.spawn(&ctx, &enemy.body.position());
        });

        self.players.iter_mut().for_each(|player| {
            bullets_hit!(player, self.enemies, 100., self.screen.height);
            player.spell.spawn(&ctx, &player.body.position());
        });

        let player_death = self.players.iter().any(|p| !p.health.is_alive());
        let enemy_death = self.enemies.iter().any(|e| !e.health.is_alive());

        check_game_finish!(
            self,
            ctx,
            players,
            player_death,
            "You died! Press R to restart."
        );

        check_game_finish!(
            self,
            ctx,
            enemies,
            enemy_death,
            "You win! Press R to restart."
        );

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
                    if let Some(elem) = self.pause_menu.front() {
                        (elem.action)(ctx, self);
                    }
                }
                _ => {}
            },
            Some(KeyCode::Escape) if !_repeated => {
                self.gamestate = GameState::Paused;
            }
            Some(KeyCode::R) if !_repeated => {
                self.restart(ctx);
            }
            Some(key) if self.gamestate == GameState::Paused => match key {
                KeyCode::Down | KeyCode::Right | KeyCode::S | KeyCode::D => {
                    if let Some(elem) = self.pause_menu.pop_front() {
                        self.pause_menu.push_back(elem);
                    }
                }
                KeyCode::Up | KeyCode::Left | KeyCode::W | KeyCode::A => {
                    if let Some(elem) = self.pause_menu.pop_back() {
                        self.pause_menu.push_front(elem);
                    }
                }
                _ => {}
            },
            _ => {}
        }

        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) -> GameResult {
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

        if let Some(background) = &self.background {
            let (w, h) = (background.width() as f32, background.height() as f32);
            canvas.draw(
                background,
                DrawParam::default().scale([width / w, height / h]),
            );
        }

        self.players.iter().for_each(|player| {
            self.draw_body(&mut canvas, &player.body, 0.12, Color::WHITE);

            player.spell.for_each_visible(|bullet| {
                self.draw_body(&mut canvas, &bullet.body, 0.05, Color::CYAN);
            });
        });

        self.enemies.iter().for_each(|enemy| {
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

            draw_at!(canvas, &healthbar, (enemy.x() - 50.0, enemy.y() - 90.0));
        });

        self.particles.iter().for_each(|particle| {
            self.draw_body(&mut canvas, &particle.bullet.body, 0.05, Color::MAGENTA);
        });

        self.texts
            .iter()
            .for_each(|text| draw_at!(canvas, text, (half_width, half_height)));

        if let Some(line) = self.story.last() {
            draw_at!(canvas, &line.text, (half_width, half_height));
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

            if let Some(elem) = self.pause_menu.front() {
                let Point2 { x, y } = elem.pos;
                draw_at!(
                    canvas,
                    &elem.img,
                    (half_width + x, half_height + y),
                    elem.select_color
                );
            }

            self.pause_menu.iter().skip(1).for_each(|elem| {
                let Point2 { x, y } = elem.pos;
                draw_at!(
                    canvas,
                    &elem.img,
                    (half_width + x, half_height + y),
                    elem.color
                );
            });
        }

        canvas.finish(ctx)
    }
}

fn load_image(ctx: &Context, path: &str) -> Image {
    Image::from_path(ctx, path).expect("Failed to load image")
}

fn main() -> GameResult {
    let (ctx, event_loop) = ContextBuilder::new("Touhou Engine", "Rontero")
        .add_resource_path(std::path::PathBuf::from("./assets"))
        .default_conf(conf::Conf::new())
        .build()?;

    let state = State::new(&ctx);
    event::run(ctx, event_loop, state);
}
