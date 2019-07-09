extern crate quicksilver;
extern crate rand;
extern crate specs;

use quicksilver::prelude::*;
use quicksilver::{geom, graphics, lifecycle};

//type V2 = vector2d::Vector2D<f32>;
type V2 = quicksilver::geom::Vector;

#[macro_use]
extern crate specs_derive;
use specs::Component;

#[derive(Debug, Component)]
struct Player {
    score: u32,
    health: u32,
    // last_bullet: u32,
}

#[derive(Debug, Component)]
struct Asteroid;

#[derive(Debug, Component)]
struct Bullet;

#[derive(Debug, Copy, Clone, Component)]
struct RigidBody {
    x: V2,
    v: V2,
    phi: f32,
    omega: f32,
}

#[derive(Debug, Component)]
struct Rectangle {
    w: f32,
    h: f32,
}

#[derive(Debug, Component)]
struct Polygon {
    pts: Vec<V2>,
}

impl Polygon {
    fn new(x: Vec<f32>, y: Vec<f32>) -> Polygon {
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

    fn random() -> Polygon {
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

    fn len(&self) -> usize {
        self.pts.len() - 1
    }

    fn area(&self) -> f32 {
        let mut a: f32 = 0.0;
        for k in 0..self.len() {
            a += 0.5 * (self.pts[k].x * self.pts[k + 1].y - self.pts[k + 1].x * self.pts[k].y);
        }
        a.abs()
    }

    fn scale(&mut self, scale: f32) {
        for k in 0..(self.len() + 1) {
            self.pts[k] *= scale;
        }
    }
}

#[derive(Debug, Component)]
struct Color {
    color: graphics::Color,
}
