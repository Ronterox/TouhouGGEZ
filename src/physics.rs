use ggez::mint::Point2;

pub trait Movable {
    fn get_rigidbody(&self) -> &Rigidbody;
    fn set_position(&mut self, x: f32, y: f32);
    fn speed(&self) -> f32 {
        let Point2 { x, y } = self.get_rigidbody().velocity;
        x.max(y)
    }
    fn velocity(&self) -> &Point2<f32> {
        &self.get_rigidbody().velocity
    }
    fn position(&self) -> &Point2<f32> {
        &self.get_rigidbody().position
    }
    fn x(&self) -> f32 {
        self.position().x
    }
    fn y(&self) -> f32 {
        self.position().y
    }
    fn move_by(&mut self, Point2 { x, y }: Point2<f32>) {
        self.set_position(self.x() + x, self.y() + y);
    }
}


#[derive(Clone)]
pub struct Rigidbody {
    pub position: Point2<f32>,
    pub velocity: Point2<f32>,
}

