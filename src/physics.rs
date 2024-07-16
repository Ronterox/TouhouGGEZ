use ggez::mint::Point2;

pub trait Movable {
    fn get_rigidbody(&self) -> &Rigidbody;
    fn set_position(&mut self, x: f32, y: f32);
    fn speed(&self) -> f32 {
        self.get_rigidbody().speed
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
    fn move_by(&mut self, x: f32, y: f32) {
        let spd = self.speed();
        self.set_position(self.x() + x * spd, self.y() + y * spd);
    }
}


pub struct Rigidbody {
    pub position: Point2<f32>,
    pub speed: f32,
}

