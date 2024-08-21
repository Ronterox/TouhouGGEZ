mod physics;

use std::{
    collections::{HashMap, VecDeque},
    str::FromStr,
};

use ggez::{graphics::*, input::keyboard::KeyCode, *};
use mint::Point2;
use physics::{Movable, Rigidbody};

#[derive(Clone)]
struct Body {
    rigidbody: Rigidbody,
    sprite: Image,
}

#[derive(Clone)]
struct Bullet {
    body: Body,
    is_visible: bool,
    direction: (f32, f32),
}

struct Spell {
    bullets: Vec<Bullet>,
    shot_timer: Timer,
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

type StoryLine = (Text, Sprite, Point2<f32>, std::time::Duration);

struct Story {
    lines: Vec<StoryLine>,
}

struct Timer {
    time: std::time::Duration,
    delay: f32,
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

type UIText = (Text, Point2<f32>);

struct State {
    gamestate: GameState,
    pause_menu: VecDeque<UIText>,
    story: Story,

    win_w: f32,
    win_h: f32,

    texts: Vec<Text>,
    players: Vec<Player>,
    enemies: Vec<Enemy>,
    particles: Vec<Particle>,
    background: Option<Image>,
}

trait Distance {
    fn distance(&self, other: &Self) -> f32;
}

impl Particle {
    fn new(ctx: &Context, ttl: f32, position: [f32; 2], speed: f32, direction: (f32, f32)) -> Self {
        let mut bullet = Bullet::new(ctx, speed, direction);
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
            self.bullet.body.move_by(self.bullet.direction);
        }
    }
}

impl Health {
    fn take_damage(&mut self, damage: u32) -> bool {
        self.health = self.health.saturating_sub(damage);
        (self.on_hit)(self.health);

        self.health == 0
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
    fn new(speed: f32, position: [f32; 2], ctx: &Context, image_path: &str) -> Self {
        Self {
            rigidbody: Rigidbody {
                position: Point2::from(position),
                speed,
            },
            sprite: load_image(ctx, image_path),
        }
    }
}

impl Bullet {
    fn new(ctx: &Context, speed: f32, direction: (f32, f32)) -> Self {
        Self {
            body: Body::new(speed, [0.0, 0.0], ctx, BULLET_IMG_PATH),
            is_visible: false,
            direction,
        }
    }

    fn update(&mut self) {
        self.body.move_by(self.direction);
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
            body: Body::new(10.0, [350.0, 350.0], ctx, PLAYER_IMG_PATH),
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
            body: Body::new(speed, [350.0, 100.0], ctx, ENEMY_IMG_PATH),
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
        self.body.move_by((*vel, 0.0));
    }
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

impl State {
    fn new(ctx: &Context) -> Self {
        let (vars, _) = touhoulang::parse_text(&std::fs::read_to_string("script.touhou").unwrap());
        dbg!(&vars);

        let bullet = |speed: f32, direction: (f32, f32)| Bullet::new(ctx, speed, direction);

        let mut players = vec![];
        let player = vars.get("player").map(|player| {
            Player::new(
                ctx,
                vars_parse(&vars, player, "health", 1),
                bullet(vars_parse(&vars, player, "bullet.speed", 20.), DIR_UP),
                vars_parse(&vars, player, "bullets", 8),
            )
        });

        if let Some(player) = player {
            players.push(player);
        }

        let mut enemies = vec![];
        let enemy = vars.get("enemy").map(|enemy| {
            Enemy::new(
                ctx,
                vars_parse(&vars, enemy, "health", 200),
                vars_parse(&vars, enemy, "speed", 4.0),
                bullet(vars_parse(&vars, enemy, "bullet.speed", 5.), DIR_DOWN),
                vars_parse(&vars, enemy, "bullets", 5),
            )
        });

        if let Some(enemy) = enemy {
            enemies.push(enemy);
        }

        let background = vars
            .get("background")
            .map(|background| load_image(ctx, &background.replace('"', "/")));

        let (win_w, win_h) = ctx.gfx.size();

        let story_line = |text: &str, sprite: Sprite, pos: [f32; 2], time: u32| -> StoryLine {
            (
                centered_text(text),
                sprite,
                pos.into(),
                std::time::Duration::from_secs(time.into()),
            )
        };

        let player_spr = Sprite {
            image: load_image(ctx, PLAYER_IMG_PATH),
            color: Color::WHITE,
        };

        let enemy_spr = Sprite {
            image: load_image(ctx, ENEMY_IMG_PATH),
            color: Color::BLACK,
        };

        let mut story_lines = vec![
            story_line("The story begins...", player_spr, [0., 0.], 0),
            story_line("I'm going to kill you!", enemy_spr, [-win_w * 0.7, 0.], 2),
        ];
        story_lines.reverse();

        Self {
            gamestate: GameState::Cinematic,
            story: Story { lines: story_lines },

            pause_menu: vec![
                (centered_text("Resume"), Point2 { x: 0., y: 0. }),
                (centered_text("Quit"), Point2 { x: 0., y: 100. }),
            ]
            .into(),

            win_w,
            win_h,

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
                    .for_each(|player| player.move_by(dir));
            }
        }

        let mut enemy_death = false;
        let mut player_death = false;

        self.enemies.iter_mut().for_each(|enemy| {
            enemy.move_auto(&ctx);

            enemy.spell.for_each_visible_mut(|bullet| {
                bullet.update();

                let mut hit_player = false;
                self.players.iter_mut().for_each(|player| {
                    if bullet.collided(player.position(), 25.) {
                        player_death = player.health.take_damage(1);
                        hit_player = true;
                    }
                });

                if bullet.body.y() > 800.0 || hit_player {
                    bullet.is_visible = false;
                }
            });

            enemy.spell.spawn(&ctx, &enemy.body.position());
        });

        self.players.iter_mut().for_each(|player| {
            player.spell.for_each_visible_mut(|bullet| {
                bullet.update();

                let mut hit_enemy = false;
                self.enemies.iter_mut().for_each(|enemy| {
                    if bullet.collided(enemy.position(), 100.) {
                        enemy_death = enemy.health.take_damage(1);
                        hit_enemy = true;
                    }
                });

                if bullet.body.y() < 0.0 || hit_enemy {
                    bullet.is_visible = false;
                }
            });

            player.spell.spawn(&ctx, &player.body.position());
        });

        if player_death {
            self.players.retain(|player| {
                let is_alive = player.health.is_alive();
                if !is_alive {
                    let start_pos = [player.x(), player.y()];
                    for dir in [DIR_UP, DIR_DOWN, DIR_LEFT, DIR_RIGHT] {
                        self.particles
                            .push(Particle::new(ctx, 2.0, start_pos, 5.0, dir));
                    }
                }
                is_alive
            });

            if self.players.is_empty() {
                self.texts
                    .push(centered_text("You died! Press R to restart."));
            }
        }

        if enemy_death {
            self.enemies.retain(|enemy| enemy.health.is_alive());
            if self.enemies.is_empty() {
                self.texts
                    .push(centered_text("You win! Press R to restart."));
            }
        }

        self.particles.retain_mut(|particle| {
            particle.update(ctx);
            particle.bullet.is_visible
        });

        Ok(())
    }
}

const PLAYER_IMG_PATH: &str = "/sakuya.png";
const ENEMY_IMG_PATH: &str = "/sakuya.png";
const BULLET_IMG_PATH: &str = "/isaac.png";

const DIR_UP: (f32, f32) = (0.0, -1.0);
const DIR_DOWN: (f32, f32) = (0.0, 1.0);
const DIR_LEFT: (f32, f32) = (-1.0, 0.0);
const DIR_RIGHT: (f32, f32) = (1.0, 0.0);

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

impl ggez::event::EventHandler<GameError> for State {
    fn resize_event(
        &mut self,
        _ctx: &mut Context,
        _width: f32,
        _height: f32,
    ) -> Result<(), GameError> {
        self.win_w = _width;
        self.win_h = _height;
        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        input: input::keyboard::KeyInput,
        _repeated: bool,
    ) -> Result<(), GameError> {
        match input.keycode {
            Some(KeyCode::Return) if !_repeated && self.gamestate == GameState::Cinematic => {
                if let Some(_line) = self.story.lines.pop() {
                    if self.story.lines.is_empty() {
                        self.gamestate = GameState::Combat;
                    }
                }
            }
            Some(KeyCode::Return) if !_repeated && self.gamestate == GameState::Paused => {
                if let Some((text, _)) = self.pause_menu.front() {
                    if let Some(fragment) = text.fragments().first() {
                        if fragment.text == "Resume" {
                            self.gamestate = GameState::Cinematic;
                        } else if fragment.text == "Quit" {
                            ctx.request_quit();
                        }
                    }
                }
            }
            Some(KeyCode::Escape) if !_repeated => {
                self.gamestate = GameState::Paused;
            }
            Some(KeyCode::Q) if !_repeated => ctx.request_quit(),
            Some(KeyCode::R) if !_repeated => {
                println!("Game Restarted!");
                *self = State::new(&ctx);
            }
            Some(key) if self.gamestate == GameState::Paused => {
                if key == KeyCode::Down {
                    if let Some(elem) = self.pause_menu.pop_front() {
                        self.pause_menu.push_back(elem);
                    }
                }
            }
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

        if let Some(background) = &self.background {
            let (w, h) = (background.width() as f32, background.height() as f32);
            canvas.draw(
                background,
                DrawParam::default().scale([self.win_w / w, self.win_h / h]),
            );
        }

        self.players.iter().for_each(|player| {
            self.draw_body(&mut canvas, &player.body, 0.12, Color::WHITE);

            player.spell.for_each_visible(|bullet| {
                self.draw_body(&mut canvas, &bullet.body, 0.05, Color::CYAN);
            });

            let circle = Mesh::new_circle(
                ctx,
                DrawMode::fill(),
                *player.body.position(),
                10.,
                0.1,
                Color::from_rgba(255, 0, 0, 127),
            )
            .unwrap();

            canvas.draw(&circle, DrawParam::default());
        });

        self.enemies.iter().for_each(|enemy| {
            self.draw_body(&mut canvas, &enemy.body, 0.2, Color::BLACK);

            enemy.spell.for_each_visible(|bullet| {
                self.draw_body(&mut canvas, &bullet.body, 0.05, Color::RED);
            });

            let healthbar = Mesh::new_rectangle(
                ctx,
                DrawMode::fill(),
                Rect::new(0.0, 0.0, enemy.health.percentage() * 100., 10.0),
                Color::from_rgba(255, 0, 0, 127),
            )
            .unwrap();

            canvas.draw(
                &healthbar,
                DrawParam::default().dest([enemy.x() - 50.0, enemy.y() - 90.0]),
            );
        });

        self.particles.iter().for_each(|particle| {
            self.draw_body(&mut canvas, &particle.bullet.body, 0.05, Color::MAGENTA);
        });

        self.texts.iter().for_each(|text| {
            canvas.draw(
                text,
                DrawParam::default().dest([self.win_w * 0.5, self.win_h * 0.5]),
            );
        });

        if let Some((text, Sprite { image, color }, Point2 { x, y }, _)) = self.story.lines.last() {
            canvas.draw(
                text,
                DrawParam::default().dest([self.win_w * 0.5, self.win_h * 0.5]),
            );

            canvas.draw(
                image,
                DrawParam::default()
                    .dest([self.win_w * 0.5 + x, self.win_h * 0.5 + y])
                    .color(*color),
            );
        }

        // TODO: limited pauses, with breaking effect after unpausing
        if self.gamestate == GameState::Paused {
            let transparent_rect = Mesh::new_rectangle(
                ctx,
                DrawMode::fill(),
                Rect::new(0.0, 0.0, self.win_w, self.win_h),
                Color::from_rgba(0, 0, 0, 127),
            )?;
            canvas.draw(&transparent_rect, DrawParam::default());

            if let Some((text, Point2 { x, y })) = self.pause_menu.front() {
                canvas.draw(
                    text,
                    DrawParam::default()
                        .dest([self.win_w * 0.5 + x, self.win_h * 0.5 + y])
                        .color(Color::YELLOW),
                )
            }

            self.pause_menu
                .iter()
                .skip(1)
                .for_each(|(text, Point2 { x, y })| {
                    canvas.draw(
                        text,
                        DrawParam::default()
                            .dest([self.win_w * 0.5 + x, self.win_h * 0.5 + y])
                            .color(Color::WHITE),
                    )
                });
        }

        canvas.finish(ctx)
    }
}

fn vars_parse<T: FromStr + Default>(
    vars: &HashMap<String, String>,
    key: &str,
    params: &str,
    default: T,
) -> T {
    vars.get(format!("{key}.{params}").as_str())
        .map(|x| x.parse().unwrap_or_default())
        .unwrap_or(default)
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
