use ggez::{
    graphics::{Color, DrawParam, Text},
    mint::Point2,
    *,
};

struct State {
    dt: std::time::Duration,
}

impl ggez::event::EventHandler<GameError> for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.dt = ctx.time.delta();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);

        let text = Text::new(format!("Delta Time: {:?}", self.dt));
        let dest = DrawParam::default().dest(Point2 { x: 10.0, y: 10.0 });
        canvas.draw(&text, dest);

        canvas.finish(ctx)
    }
}

fn main() {
    let state = State {
        dt: std::time::Duration::new(0, 0),
    };

    let c = conf::Conf::new();
    let (ctx, event_loop) = ContextBuilder::new("hello_ggez", "that's me")
        .default_conf(c)
        .build()
        .unwrap();

    event::run(ctx, event_loop, state);
}
