mod physics;

use ggez::{graphics::Color, input::keyboard::KeyCode, *};
use mint::Point2;
use physics::{Movable, Rigidbody};
use touhoulang::parse_text;

type Bullet = Body;

#[derive(PartialEq, Clone)]
enum Visibility {
    Visible,
    Hidden,
}

struct Spell {
    bullets: Vec<Bullet>,
    shot_timer: Timer,
}

#[derive(Clone)]
struct Body {
    rigidbody: Rigidbody,
    sprite: graphics::Image,
    visibility: Visibility,
}

struct Enemy {
    body: Body,
    spell: Spell,
    velocities: Vec<f32>,
    move_timer: Timer,
    health: u32,
}

struct Player {
    body: Body,
    spell: Spell,
}

struct State {
    player: Option<Player>,
    enemy: Option<Enemy>,
}

struct Timer {
    time: std::time::Duration,
    delay: f32,
}

trait Distance {
    fn distance(&self, other: &Self) -> f32;
}

impl Spell {
    fn new(bullet: Bullet, bullets_size: usize, delay: f32) -> Self {
        Self {
            bullets: std::iter::repeat(bullet).take(bullets_size).collect(),
            shot_timer: Timer::new(delay),
        }
    }

    fn ready(&mut self, ctx: &Context) -> bool {
        self.shot_timer.ready(ctx)
    }

    fn for_each_visible(&self, f: impl FnMut(&Bullet)) {
        self.bullets.iter().filter(|x| x.visible()).for_each(f);
    }

    fn mut_for_each_visible(&mut self, f: impl FnMut(&mut Bullet)) {
        self.bullets.iter_mut().filter(|x| x.visible()).for_each(f);
    }

    fn find_hidden_then_do(&mut self, f: impl FnOnce(&mut Bullet)) {
        self.bullets.iter_mut().find(|x| !x.visible()).map(f);
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
            visibility: Visibility::Hidden,
        }
    }

    fn visible(&self) -> bool {
        self.visibility == Visibility::Visible
    }
}

impl Player {
    fn new(ctx: &Context, bullet: Bullet, bullets_size: usize) -> Self {
        Self {
            body: Body::new(10.0, [350.0, 350.0], ctx, "/sakuya.png"),
            spell: Spell::new(bullet, bullets_size, 0.1),
        }
    }
}

impl Enemy {
    fn new(ctx: &Context, bullet: Bullet, bullets_size: usize) -> Self {
        Self {
            health: 200,
            body: Body::new(2., [350.0, 100.0], ctx, "/sakuya.png"),
            spell: Spell::new(bullet, bullets_size, 0.5),
            velocities: vec![-1., 0., 1., 0., 1., 0., -1., 0.],
            move_timer: Timer::new(1.5),
        }
    }
}

impl State {
    fn if_press_move(&mut self, ctx: &Context, key: KeyCode, dir: (f32, f32)) {
        if let Some(player) = &mut self.player {
            if ctx.keyboard.is_key_pressed(key) {
                player.move_by(dir.0, dir.1);
            }
        }
    }

    fn new(ctx: &Context) -> Self {
        let (vars, _) = parse_text(&std::fs::read_to_string("script.touhou").unwrap());
        dbg!(vars);

        let bullet = |speed: f32| Bullet::new(speed, [0., 0.], ctx, "/isaac.png");

        Self {
            player: Some(Player::new(ctx, bullet(20.0), 8)),
            enemy: Some(Enemy::new(ctx, bullet(5.0), 5)),
        }
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

impl ggez::event::EventHandler<GameError> for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.if_press_move(&ctx, KeyCode::W, (0.0, -1.0));
        self.if_press_move(&ctx, KeyCode::S, (0.0, 1.0));
        self.if_press_move(&ctx, KeyCode::A, (-1.0, 0.0));
        self.if_press_move(&ctx, KeyCode::D, (1.0, 0.0));

        if let (Some(player), Some(enemy)) = (&mut self.player, &mut self.enemy) {
            let player_pos = player.position();
            let enemy_pos = enemy.position().to_owned();

            enemy.spell.mut_for_each_visible(|bullet| {
                bullet.move_by(0.0, 1.0);
                if bullet.y() > 800.0 {
                    bullet.visibility = Visibility::Hidden;
                } else if bullet.position().distance(&player_pos) < 25. {
                    todo!("You lost, player defeated!");
                }
            });

            player.spell.mut_for_each_visible(|bullet| {
                bullet.move_by(0.0, -1.0);
                let hit_enemy = bullet.position().distance(&enemy_pos) < 100.;

                if bullet.y() < 0.0 || hit_enemy {
                    bullet.visibility = Visibility::Hidden;
                }

                if hit_enemy {
                    enemy.health -= 1;
                    if enemy.health == 0 {
                        todo!("You won, enemy defeated!");
                    }

                    if enemy.health % 5 == 0 {
                        println!("Enemy health: {}", enemy.health);
                    }
                }
            });

            if player.spell.ready(&ctx) {
                let (x, y) = (player.x(), player.y());

                player.spell.find_hidden_then_do(|bullet| {
                    bullet.set_position(x, y);
                    bullet.visibility = Visibility::Visible;
                });
            }

            if enemy.spell.ready(&ctx) {
                let (x, y) = (enemy.x(), enemy.y());

                enemy.spell.find_hidden_then_do(|bullet| {
                    bullet.set_position(x, y);
                    bullet.visibility = Visibility::Visible;
                });
            }

            if enemy.move_timer.ready(&ctx) {
                enemy.velocities.rotate_left(1);
            }

            let vel = enemy.velocities.first().unwrap_or(&0.0);
            enemy.move_by(vel * enemy.speed(), 0.0);
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::from_rgb(0x2b, 0x2c, 0x2f));

        let mut draw_on_pos = |body: &Body, size: f32, color: Color| {
            canvas.draw(
                &body.sprite,
                graphics::DrawParam::new()
                    .dest(*body.position())
                    .scale([size, size])
                    .color(color)
                    .offset([0.5, 0.5]),
            );
        };

        if let Some(player) = &self.player {
            player.spell.for_each_visible(|bullet| {
                draw_on_pos(&bullet, 0.05, Color::RED);
            });

            draw_on_pos(&player.body, 0.12, Color::WHITE);

            // let circle = graphics::Mesh::new_circle(
            //     ctx,
            //     graphics::DrawMode::fill(),
            //     *player.body.position(),
            //     10.,
            //     0.1,
            //     Color::from_rgba(255, 0, 0, 127),
            // )?;
            //
            // canvas.draw(&circle, graphics::DrawParam::default());
        }

        if let Some(enemy) = &self.enemy {
            enemy.spell.for_each_visible(|bullet| {
                draw_on_pos(&bullet, 0.1, Color::RED);
            });

            draw_on_pos(&enemy.body, 0.2, Color::BLACK);
        }

        canvas.finish(ctx)
    }
}

fn load_image(ctx: &Context, path: &str) -> graphics::Image {
    graphics::Image::from_path(ctx, path).expect("Failed to load image")
}

fn main() -> GameResult {
    let (ctx, event_loop) = ContextBuilder::new("Touhou Engine", "Rontero")
        .add_resource_path(std::path::PathBuf::from("./assets"))
        .default_conf(conf::Conf::new())
        .build()?;

    let state = State::new(&ctx);
    event::run(ctx, event_loop, state);
}
