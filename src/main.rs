mod physics;

use std::{collections::HashMap, str::FromStr};

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

struct State {
    player: Option<Player>,
    enemy: Option<Enemy>,
    background: Option<Image>,
    particles: Vec<Particle>,
    texts: Vec<Text>,
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

        let player = vars.get("player").map(|player| {
            Player::new(
                ctx,
                vars_parse(&vars, player, "health", 1),
                bullet(vars_parse(&vars, player, "bullet.speed", 20.), DIR_UP),
                vars_parse(&vars, player, "bullets", 8),
            )
        });

        let enemy = vars.get("enemy").map(|enemy| {
            Enemy::new(
                ctx,
                vars_parse(&vars, enemy, "health", 200),
                vars_parse(&vars, enemy, "speed", 4.0),
                bullet(vars_parse(&vars, enemy, "bullet.speed", 5.), DIR_DOWN),
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
            particles: vec![],
            texts: vec![],
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
                    bullet.update();
                    let hit_player = bullet.collided(&player_pos, 25.);

                    if bullet.body.y() > 800.0 || hit_player {
                        bullet.is_visible = false;
                    }

                    if hit_player {
                        player_died = player.health.take_damage(1);
                    }
                });

                player.spell.mut_for_each_visible(|bullet| {
                    bullet.update();
                    let hit_enemy = bullet.collided(&enemy_pos, 100.);

                    if bullet.body.y() < 0.0 || hit_enemy {
                        bullet.is_visible = false;
                    }

                    if hit_enemy {
                        enemy_died = enemy.health.take_damage(1);
                        let enemy_pos = [enemy.x(), enemy.y()];
                        let dir = (enemy.directions[1], enemy.directions[0]);
                        self.particles
                            .push(Particle::new(&ctx, 0.5, enemy_pos, 2., dir));
                    }
                });

                player.spell.spawn(&ctx, &player.body.position());
                enemy.spell.spawn(&ctx, &enemy.body.position());
            }
            (Some(player), None) => {
                player.spell.mut_for_each_visible(|bullet| {
                    bullet.update();
                    if bullet.body.y() < 0.0 {
                        bullet.is_visible = false;
                    }
                });

                player.spell.spawn(&ctx, &player.body.position());
            }
            (None, Some(enemy)) => {
                enemy.move_auto(&ctx);

                enemy.spell.mut_for_each_visible(|bullet| {
                    bullet.update();
                    if bullet.body.y() > 800.0 {
                        bullet.is_visible = false;
                    }
                });

                enemy.spell.spawn(&ctx, &enemy.body.position());
            }
            (None, None) => {}
        }

        if player_died {
            let player = self.player.as_mut().unwrap();
            let start_pos = [player.x(), player.y()];
            for dir in [DIR_UP, DIR_DOWN, DIR_LEFT, DIR_RIGHT] {
                self.particles
                    .push(Particle::new(ctx, 2.0, start_pos, 5.0, dir));
            }
            self.player = None;
            self.texts.push(centered_text("You died! Press R to restart."));
        }

        if enemy_died {
            self.enemy = None;
            self.texts.push(centered_text("You win! Press R to restart."));
        }

        let mut remove = false;
        self.particles.iter_mut().for_each(|particle| {
            particle.update(ctx);
            if !particle.bullet.is_visible {
                remove = true;
            }
        });

        if remove {
            self.particles.retain(|particle| particle.bullet.is_visible);
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = Canvas::from_frame(ctx, Color::from_rgb(0x2b, 0x2c, 0x2f));
        let (win_w, win_h) = ctx.gfx.size();

        if let Some(background) = &self.background {
            let (w, h) = (background.width() as f32, background.height() as f32);
            canvas.draw(
                background,
                DrawParam::default().scale([win_w / w, win_h / h]),
            );
        }

        if let Some(player) = &self.player {
            player.spell.for_each_visible(|bullet| {
                self.draw_body(&mut canvas, &bullet.body, 0.05, Color::CYAN);
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
                self.draw_body(&mut canvas, &bullet.body, 0.1, Color::RED);
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

        self.particles.iter().for_each(|particle| {
            self.draw_body(&mut canvas, &particle.bullet.body, 0.05, Color::MAGENTA);
        });

        self.texts.iter().for_each(|text| {
            canvas.draw(text, DrawParam::default().dest([win_w / 2.0, win_h / 2.0]));
        });

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
