mod physics;

use std::{collections::HashMap, str::FromStr};

use ggez::{graphics::*, input::keyboard::KeyCode, *};
use mint::Point2;
use physics::{Movable, Rigidbody};

type Bullet = Body;

struct Spell {
    bullets: Vec<Bullet>,
    shot_timer: Timer,
}

#[derive(Clone)]
struct Body {
    rigidbody: Rigidbody,
    sprite: Image,
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

struct Timer {
    time: std::time::Duration,
    delay: f32,
}

struct State {
    player: Option<Player>,
    enemy: Option<Enemy>,
    background: Option<Image>,
}

struct Health {
    health: u32,
    max_health: u32,
    on_hit: fn(u32),
}

trait Distance {
    fn distance(&self, other: &Self) -> f32;
}

impl Health {
    fn take_damage(&mut self, damage: u32) -> bool {
        self.health = self.health.saturating_sub(damage);
        (self.on_hit)(self.health);

        self.health == 0
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
                    bullet.set_position(position.x, position.y);
                    bullet.is_visible = true;
                });
        }
    }

    fn for_each_visible(&self, f: impl FnMut(&Bullet)) {
        self.bullets.iter().filter(|x| x.is_visible).for_each(f);
    }

    fn mut_for_each_visible(&mut self, f: impl FnMut(&mut Bullet)) {
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
            is_visible: false,
        }
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

        let bullet = |speed: f32| Bullet::new(speed, [0., 0.], ctx, BULLET_IMG_PATH);

        let player = vars.get("player").map(|player| {
            Player::new(
                ctx,
                vars_parse(&vars, player, "health", 1),
                bullet(vars_parse(&vars, player, "bullet.speed", 20.)),
                vars_parse(&vars, player, "bullets", 8),
            )
        });

        let enemy = vars.get("enemy").map(|enemy| {
            Enemy::new(
                ctx,
                vars_parse(&vars, enemy, "health", 200),
                vars_parse(&vars, enemy, "speed", 4.0),
                bullet(vars_parse(&vars, enemy, "bullet.speed", 5.)),
                vars_parse(&vars, enemy, "bullets", 5),
            )
        });

        let background = vars
            .get("background")
            .map(|background| load_image(ctx, &background.replace('"', "/")));

        Self {
            player,
            enemy,
            background,
        }
    }

    fn if_press_move(&mut self, ctx: &Context, key: KeyCode, dir: (f32, f32)) {
        if let Some(player) = &mut self.player {
            if ctx.keyboard.is_key_pressed(key) {
                player.move_by(dir);
            }
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
}

const PLAYER_IMG_PATH: &str = "/sakuya.png";
const ENEMY_IMG_PATH: &str = "/sakuya.png";
const BULLET_IMG_PATH: &str = "/isaac.png";

const DIR_UP: (f32, f32) = (0.0, -1.0);
const DIR_DOWN: (f32, f32) = (0.0, 1.0);
const DIR_LEFT: (f32, f32) = (-1.0, 0.0);
const DIR_RIGHT: (f32, f32) = (1.0, 0.0);

impl ggez::event::EventHandler<GameError> for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.if_press_move(&ctx, KeyCode::W, DIR_UP);
        self.if_press_move(&ctx, KeyCode::S, DIR_DOWN);
        self.if_press_move(&ctx, KeyCode::A, DIR_LEFT);
        self.if_press_move(&ctx, KeyCode::D, DIR_RIGHT);

        if ctx.keyboard.is_key_pressed(KeyCode::R) {
            println!("Game Restarted!");
            *self = State::new(&ctx);
        }

        let mut player_died = false;
        let mut enemy_died = false;
        match (&mut self.player, &mut self.enemy) {
            (Some(player), Some(enemy)) => {
                enemy.move_auto(&ctx);

                let player_pos = player.position().to_owned();
                let enemy_pos = enemy.position().to_owned();

                enemy.spell.mut_for_each_visible(|bullet| {
                    bullet.move_by(DIR_DOWN);
                    let hit_player = bullet.position().distance(&player_pos) < 25.;

                    if bullet.y() > 800.0 || hit_player {
                        bullet.is_visible = false;
                    }

                    if hit_player {
                        player_died = player.health.take_damage(1);
                    }
                });

                player.spell.mut_for_each_visible(|bullet| {
                    bullet.move_by(DIR_UP);
                    let hit_enemy = bullet.position().distance(&enemy_pos) < 100.;

                    if bullet.y() < 0.0 || hit_enemy {
                        bullet.is_visible = false;
                    }

                    if hit_enemy {
                        enemy_died = enemy.health.take_damage(1);
                    }
                });

                player.spell.spawn(&ctx, &player.body.position());
                enemy.spell.spawn(&ctx, &enemy.body.position());
            }
            (Some(player), None) => {
                player.spell.mut_for_each_visible(|bullet| {
                    bullet.move_by(DIR_UP);
                    if bullet.y() < 0.0 {
                        bullet.is_visible = false;
                    }
                });

                player.spell.spawn(&ctx, &player.body.position());
            }
            (None, Some(enemy)) => {
                enemy.move_auto(&ctx);

                enemy.spell.mut_for_each_visible(|bullet| {
                    bullet.move_by(DIR_DOWN);
                    if bullet.y() > 800.0 {
                        bullet.is_visible = false;
                    }
                });

                enemy.spell.spawn(&ctx, &enemy.body.position());
            }
            (None, None) => {}
        }

        if player_died {
            self.player = None;
        }

        if enemy_died {
            self.enemy = None;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = Canvas::from_frame(ctx, Color::from_rgb(0x2b, 0x2c, 0x2f));

        if let Some(background) = &self.background {
            let (w, h) = (background.width() as f32, background.height() as f32);
            let (win_w, win_h) = ctx.gfx.size();
            canvas.draw(
                background,
                DrawParam::default().scale([win_w / w, win_h / h]),
            );
        }

        if let Some(player) = &self.player {
            player.spell.for_each_visible(|bullet| {
                self.draw_body(&mut canvas, &bullet, 0.05, Color::CYAN);
            });

            self.draw_body(&mut canvas, &player.body, 0.12, Color::WHITE);
            let circle = Mesh::new_circle(
                ctx,
                DrawMode::fill(),
                *player.body.position(),
                10.,
                0.1,
                Color::from_rgba(255, 0, 0, 127),
            )?;

            canvas.draw(&circle, DrawParam::default());
        }

        if let Some(enemy) = &self.enemy {
            enemy.spell.for_each_visible(|bullet| {
                self.draw_body(&mut canvas, &bullet, 0.1, Color::RED);
            });

            self.draw_body(&mut canvas, &enemy.body, 0.2, Color::BLACK);

            let healthbar = Mesh::new_rectangle(
                ctx,
                DrawMode::fill(),
                Rect::new(0.0, 0.0, enemy.health.percentage() * 100., 10.0),
                Color::from_rgba(255, 0, 0, 127),
            )?;

            canvas.draw(
                &healthbar,
                DrawParam::default().dest([enemy.x() - 50.0, enemy.y() - 90.0]),
            );
        }

        canvas.finish(ctx)
    }
}

fn vars_parse<T: FromStr + Default>(
    vars: &HashMap<String, String>,
    key: &str,
    params: &str,
    unless: T,
) -> T {
    vars.get(format!("{key}.{params}").as_str())
        .map(|x| x.parse().unwrap_or_default())
        .unwrap_or(unless)
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
