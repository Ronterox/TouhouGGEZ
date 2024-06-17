mod physics;

use ggez::{graphics::Color, input::keyboard::KeyCode, *};
use mint::{Point2, Vector2};
use physics::{Movable, Rigidbody};

type Bullet = Entity;
type Player = Entity;

struct Entity {
    rigidbody: Rigidbody,
    sprite: graphics::Image,
}

struct State {
    player: Player,
    bullets: Vec<Bullet>,
    bullet_timer: std::time::Duration,
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

        self.bullets.iter_mut().for_each(|bullet| {
            bullet.move_by(0.0, -1.0);
        });

        self.bullet_timer += ctx.time.delta();
        if self.bullet_timer.as_secs_f32() > 0.5 {
            self.bullets.push(Entity {
                rigidbody: Rigidbody {
                    position: Vector2 {
                        x: self.player.x(),
                        y: self.player.y(),
                    },
                    speed: 5.0,
                },
                sprite: graphics::Image::from_path(ctx, "/isaaac.jpg")
                    .expect("Failed to load image"),
            });
            self.bullet_timer = std::time::Duration::new(0, 0);
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);

        let mut draw_on_pos = |sprite: &graphics::Image, x: f32, y: f32| {
            canvas.draw(
                sprite,
                graphics::DrawParam::new()
                    .dest(Point2 { x, y })
                    .scale(Point2 { x: 0.2, y: 0.2 }),
            );
        };

        self.bullets.iter().for_each(|bullet| {
            draw_on_pos(&bullet.sprite, bullet.x(), bullet.y());
        });

        draw_on_pos(&self.player.sprite, self.player.x(), self.player.y());

        canvas.finish(ctx)
    }
}

fn init(ctx: &Context) -> State {
    let load_image =
        |path: &str| graphics::Image::from_path(ctx, path).expect("Failed to load image");

    let state = State {
        player: Entity {
            rigidbody: Rigidbody {
                position: Vector2 { x: 100.0, y: 100.0 },
                speed: 5.0,
            },
            sprite: load_image("/sakuya.png"),
        },
        bullets: vec![],
        bullet_timer: std::time::Duration::new(0, 0),
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
