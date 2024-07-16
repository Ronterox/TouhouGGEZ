mod physics;

use ggez::{graphics::Color, input::keyboard::KeyCode, *};
use mint::Point2;
use physics::{Movable, Rigidbody};

type Bullet = Body;

#[derive(PartialEq)]
enum Visibility {
    Visible,
    Hidden,
}

struct Spell {
    bullets: Vec<Bullet>,
    shot_timer: Timer,
}

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
    player: Player,
    enemy: Enemy,
}

struct Timer {
    time: std::time::Duration,
    delay: f32,
}

trait Distance {
    fn distance(&self, other: &Self) -> f32;
}

impl Distance for Point2<f32> {
    fn distance(&self, other: &Self) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

impl Spell {
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
        Timer {
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
    fn visible(&self) -> bool {
        self.visibility == Visibility::Visible
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
    fn if_press_move(&mut self, ctx: &Context, key: KeyCode, dir: (f32, f32)) {
        if ctx.keyboard.is_key_pressed(key) {
            self.player.move_by(dir.0, dir.1);
        }
    }
}

impl ggez::event::EventHandler<GameError> for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.if_press_move(&ctx, KeyCode::W, (0.0, -1.0));
        self.if_press_move(&ctx, KeyCode::S, (0.0, 1.0));
        self.if_press_move(&ctx, KeyCode::A, (-1.0, 0.0));
        self.if_press_move(&ctx, KeyCode::D, (1.0, 0.0));

        let player = &mut self.player;
        let player_pos = player.position().clone();

        let enemy = &mut self.enemy;
        let enemy_pos = enemy.position().clone();

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

        enemy.spell.mut_for_each_visible(|bullet| {
            bullet.move_by(0.0, 1.0);
            if bullet.y() > 800.0 {
                bullet.visibility = Visibility::Hidden;
            } else if bullet.position().distance(&player_pos) < 25. {
                todo!("You lost, player defeated!");
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

        self.player.spell.for_each_visible(|bullet| {
            draw_on_pos(&bullet, 0.05, Color::RED);
        });

        self.enemy.spell.for_each_visible(|bullet| {
            draw_on_pos(&bullet, 0.1, Color::RED);
        });

        draw_on_pos(&self.player.body, 0.12, Color::WHITE);
        draw_on_pos(&self.enemy.body, 0.2, Color::BLACK);

        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            *self.player.body.position(),
            10.,
            0.1,
            Color::from_rgba(255, 0, 0, 127),
        )?;

        canvas.draw(&circle, graphics::DrawParam::default());

        canvas.finish(ctx)
    }
}

fn init(ctx: &Context) -> State {
    let load_image =
        |path: &str| graphics::Image::from_path(ctx, path).expect("Failed to load image");

    let mut bullets = Vec::new();
    for _ in 0..8 {
        let bullet = Bullet {
            rigidbody: Rigidbody {
                position: Point2 { x: 0., y: 0. },
                speed: 20.0,
            },
            sprite: load_image("/isaac.png"),
            visibility: Visibility::Hidden,
        };
        bullets.push(bullet);
    }

    let mut enemy_bullets = Vec::new();
    for _ in 0..8 {
        let bullet = Bullet {
            rigidbody: Rigidbody {
                position: Point2 { x: 0., y: 0. },
                speed: 5.0,
            },
            sprite: load_image("/isaac.png"),
            visibility: Visibility::Hidden,
        };
        enemy_bullets.push(bullet);
    }

    let state = State {
        player: Player {
            body: Body {
                rigidbody: Rigidbody {
                    position: Point2 { x: 350.0, y: 350.0 },
                    speed: 10.0,
                },
                sprite: load_image("/sakuya.png"),
                visibility: Visibility::Visible,
            },
            spell: Spell {
                bullets,
                shot_timer: Timer::new(0.1),
            },
        },
        enemy: Enemy {
            health: 200,
            body: Body {
                rigidbody: Rigidbody {
                    position: Point2 { x: 350.0, y: 100.0 },
                    speed: 2.0,
                },
                sprite: load_image("/sakuya.png"),
                visibility: Visibility::Visible,
            },
            spell: Spell {
                bullets: enemy_bullets,
                shot_timer: Timer::new(0.5),
            },
            velocities: vec![-1., 0., 1., 0., 1., 0., -1., 0.],
            move_timer: Timer::new(1.5),
        },
    };

    return state;
}

fn main() -> GameResult {
    let (ctx, event_loop) = ContextBuilder::new("Touhou Engine", "Rontero")
        .add_resource_path(std::path::PathBuf::from("./assets"))
        .default_conf(conf::Conf::new())
        .build()
        .unwrap();

    let state = init(&ctx);
    event::run(ctx, event_loop, state);
}
