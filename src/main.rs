use ggez::{graphics::Color, input::keyboard::KeyCode, *};
use mint::Point2;

struct State {
    x: f32,
    y: f32,
    speed: f32,
    sprite: graphics::Image,
}

impl ggez::event::EventHandler<GameError> for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if ctx.keyboard.is_key_pressed(KeyCode::W) {
            self.y -= self.speed;
        }

        if ctx.keyboard.is_key_pressed(KeyCode::S) {
            self.y += self.speed;
        }

        if ctx.keyboard.is_key_pressed(KeyCode::A) {
            self.x -= self.speed;
        }

        if ctx.keyboard.is_key_pressed(KeyCode::D) {
            self.x += self.speed;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);

        canvas.draw(
            &self.sprite,
            graphics::DrawParam::new()
                .dest(Point2 {
                    x: self.x,
                    y: self.y,
                })
                .scale(Point2 { x: 0.2, y: 0.2 }),
        );

        canvas.finish(ctx)
    }
}

fn main() -> GameResult {
    let (ctx, event_loop) = ContextBuilder::new("Touhou Engine", "Rontero")
        .add_resource_path(std::path::PathBuf::from("./assets"))
        .default_conf(conf::Conf::new())
        .build()
        .unwrap();

    let state = State {
        x: 0.0,
        y: 0.0,
        speed: 5.0,
        sprite: graphics::Image::from_path(&ctx, "/sakuya.png")?,
    };

    event::run(ctx, event_loop, state);
}
