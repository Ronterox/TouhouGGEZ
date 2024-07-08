mod physics;

use ggez::{graphics::Color, input::keyboard::KeyCode, *};
use mint::{Point2, Vector2};
use physics::{Movable, Rigidbody};

type Bullet = Entity;
type Player = Entity;

#[derive(PartialEq)]
enum Visibility {
    Visible,
    Hidden,
}

struct Entity {
    rigidbody: Rigidbody,
    sprite: graphics::Image,
    visibility: Visibility,
}

struct Enemy {
    entity: Entity,
    velocities: Vec<f32>,
    move_timer: std::time::Duration,
}

struct State {
    player: Player,
    enemy: Enemy,
    bullets: Vec<Bullet>,
    shot_timer: std::time::Duration,
}

impl Entity {
    fn visible(&self) -> bool {
        self.visibility == Visibility::Visible
    }
}

impl Movable for Entity {
    fn get_rigidbody(&self) -> &Rigidbody {
        &self.rigidbody
    }

    fn set_position(&mut self, x: f32, y: f32) {
        self.rigidbody.position.x = x;
        self.rigidbody.position.y = y;
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

        self.bullets
            .iter_mut()
            .filter(|x| x.visible())
            .for_each(|bullet| {
                bullet.move_by(0.0, -1.0);
                if bullet.y() < 0.0 {
                    bullet.visibility = Visibility::Hidden;
                }
            });

        self.shot_timer += ctx.time.delta();
        if self.shot_timer.as_secs_f32() > 0.1 {
            self.bullets
                .iter_mut()
                .find(|x| !x.visible())
                .map(|bullet| {
                    bullet.set_position(self.player.x(), self.player.y());
                    bullet.visibility = Visibility::Visible;
                });
            self.shot_timer = std::time::Duration::new(0, 0);
        }

        self.enemy.move_timer += ctx.time.delta();
        if self.enemy.move_timer.as_secs_f32() > 1.5 {
            self.enemy.velocities.rotate_left(1);
            self.enemy.move_timer = std::time::Duration::new(0, 0);
        }
        let vel = self.enemy.velocities.first().unwrap_or(&0.0);
        self.enemy.entity.move_by(vel * self.enemy.entity.speed(), 0.0);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::from_rgb(0x2b, 0x2c, 0x2f));

        let mut draw_on_pos = |entity: &Entity, size: f32, color: Color| {
            canvas.draw(
                &entity.sprite,
                graphics::DrawParam::new()
                    .dest(*entity.position())
                    .scale(Point2 { x: size, y: size })
                    .color(color),
            );
        };

        self.bullets
            .iter()
            .filter(|x| x.visible())
            .for_each(|bullet| {
                draw_on_pos(&bullet, 0.05, Color::RED);
            });

        draw_on_pos(&self.player, 0.2, Color::WHITE);
        draw_on_pos(&self.enemy.entity, 0.15, Color::BLACK);

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
                position: Vector2 { x: 0., y: 0. },
                speed: 20.0,
            },
            sprite: load_image("/isaac.png"),
            visibility: Visibility::Hidden,
        };
        bullets.push(bullet);
    }

    let enemy = Enemy {
        entity: Entity {
            rigidbody: Rigidbody {
                position: Vector2 { x: 350.0, y: 100.0 },
                speed: 2.0,
            },
            sprite: load_image("/sakuya.png"),
            visibility: Visibility::Visible,
        },
        velocities: vec![-1., 0., 1., 0., 1., 0., -1., 0.],
        move_timer: std::time::Duration::new(0, 0),
    };

    let state = State {
        player: Player {
            rigidbody: Rigidbody {
                position: Vector2 { x: 350.0, y: 350.0 },
                speed: 10.0,
            },
            sprite: load_image("/sakuya.png"),
            visibility: Visibility::Visible,
        },
        enemy,
        bullets,
        shot_timer: std::time::Duration::new(0, 0),
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
