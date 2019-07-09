use quicksilver::graphics;
use specs::{Component, DenseVecStorage};

pub type V2 = quicksilver::geom::Vector;

#[derive(Debug, Component)]
pub struct Player {
    pub score: u32,
    pub health: u32,
    // last_bullet: u32,
}

#[derive(Debug, Component)]
pub struct Asteroid;

#[derive(Debug, Component)]
pub struct Bullet;

#[derive(Debug, Copy, Clone, Component)]
pub struct RigidBody {
    pub x: V2,
    pub v: V2,
    pub phi: f32,
    pub omega: f32,
}

#[derive(Debug, Component)]
pub struct Rectangle {
    pub w: f32,
    pub h: f32,
}

#[derive(Debug, Component)]
pub struct Color {
    pub color: graphics::Color,
}

#[derive(Debug, Component)]
pub struct Polygon {
    pub pts: Vec<V2>,
}

impl Polygon {
    pub fn new(x: Vec<f32>, y: Vec<f32>) -> Polygon {
        // find the polygon's barycenter
        let mut area: f32 = 0.0;
        for k in 0..(x.len() - 1) {
            area += 0.5 * (x[k] * y[k + 1] - x[k + 1] * y[k]);
        }

        let mut cx: f32 = 0.0;
        let mut cy: f32 = 0.0;

        let scale: f32 = 6.0 * area;

        for k in 0..(x.len() - 1) {
            let b = (x[k] * y[k + 1] - x[k + 1] * y[k]) / scale;
            cx += b * (x[k] + x[k + 1]);
            cy += b * (y[k] + y[k + 1]);
        }

        let mut p = Vec::<V2>::new();
        for k in 0..x.len() {
            p.push(V2::new(x[k] - cx, y[k] - cy));
        }
        p.push(p[0].clone()); // close the loop
        Polygon { pts: p }
    }

    pub fn random() -> Polygon {
        use crate::rand::Rng;
        let mut rng = rand::thread_rng();
        let mut x = Vec::<f32>::new();
        let mut y = Vec::<f32>::new();

        let n: usize = rng.gen_range(3usize, 12usize);

        for k in 0..n {
            let phi = (k as f32) * 2.0 * std::f32::consts::PI / ((n + 1) as f32);
            let r = 1.0 + 14.0 * rng.gen::<f32>();
            x.push(r * phi.cos());
            y.push(r * phi.sin());
        }
        Polygon::new(x, y)
    }

    pub fn len(&self) -> usize {
        self.pts.len() - 1
    }

    pub fn area(&self) -> f32 {
        let mut a: f32 = 0.0;
        for k in 0..self.len() {
            a += 0.5 * (self.pts[k].x * self.pts[k + 1].y - self.pts[k + 1].x * self.pts[k].y);
        }
        a.abs()
    }

    pub fn scale(&mut self, scale: f32) {
        for k in 0..(self.len() + 1) {
            self.pts[k] *= scale;
        }
    }
}
