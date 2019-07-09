use super::components::*;
use quicksilver::graphics;

// pub type V2 = quicksilver::geom::Vector;

pub fn euclidean(v: &V2, phi: f32, dv: &V2) -> V2 {
    let c = phi.cos();
    let s = phi.sin();
    V2::new(dv.x + c * v.x - s * v.y, dv.y + s * v.x + c * v.y)
}

pub struct PhysicsUpdate;
pub struct BulletAsteroidCollision;

impl<'a> specs::System<'a> for PhysicsUpdate {
    type SystemData = specs::WriteStorage<'a, RigidBody>;
    fn run(&mut self, mut state: Self::SystemData) {
        use specs::Join;
        let dt: f32 = 0.06;

        for state in (&mut state).join() {
            let vs = state.v * dt;
            state.x += vs; // state.v;
            let mut flipx = false;
            if state.x.y < 0.0 {
                state.x.y += 100.0;
                flipx = true;
            }
            if state.x.y > 100.0 {
                state.x.y = 100.0 - state.x.y;
                flipx = true;
            }
            if state.x.x < 0.0 {
                state.x.x += 100.0;
                state.x.y = 100.0 - state.x.y;
            }
            if state.x.x > 100.0 {
                state.x.x = 100.0 - state.x.x;
                state.x.y = 100.0 - state.x.y;
            }
            if flipx {
                state.x.x = 100.0 - state.x.x;
            }
            state.phi += dt * state.omega;
        }
    }
}

impl<'a> specs::System<'a> for BulletAsteroidCollision {
    type SystemData = (
        specs::Entities<'a>,
        specs::ReadStorage<'a, RigidBody>,
        specs::ReadStorage<'a, Polygon>,
        specs::ReadStorage<'a, Asteroid>,
        specs::ReadStorage<'a, Bullet>,
        specs::Read<'a, specs::LazyUpdate>,
    );
    fn run(
        &mut self,
        (entities, rb_read, poly_read, ast_read, bul_read, updater): Self::SystemData,
    ) {
        //TODO: refactor this. this is ridiculous.
        use specs::Join;
        for (ent_bullet, rb_bullet, _bullet) in (&entities, &rb_read, &bul_read).join() {
            for (ent_ast, rb_ast, ast_poly, _ast) in
                (&entities, &rb_read, &poly_read, &ast_read).join()
            {
                // For each line segment in the polygon,
                // compute the distance from the bullet to the line segment
                for k in 0..ast_poly.len() {
                    let x0 = euclidean(&ast_poly.pts[k + 0], rb_ast.phi, &rb_ast.x);
                    let x1 = euclidean(&ast_poly.pts[k + 1], rb_ast.phi, &rb_ast.x);

                    let l2 = (x1 - x0).len2();
                    let t = (rb_bullet.x - x0).dot(x1 - x0) / l2;
                    let p = x0 * (1.0 - t) + x1 * t;

                    let d = p.distance(rb_bullet.x);

                    if d < 0.5 && t > 0.0 && t < 1.0 {
                        // Call this a collision
                        entities.delete(ent_bullet);
                        entities.delete(ent_ast);

                        // Get area
                        let mut a = ast_poly.area();
                        // kill small asteroids
                        if a > 10.0 {
                            use crate::rand::Rng;
                            a *= 0.95; // Decrease mass slightly
                            let mut rng = rand::thread_rng();
                            // Momentum
                            let m = rb_ast.v * a + rb_bullet.v * 0.5;

                            // Make two asteroids. We conserve "mass" (area),
                            // momentum, and angular momentum
                            let a1 = rng.gen::<f32>() * a;
                            let a2 = a - a1;

                            // Random direction
                            let rb1 = RigidBody {
                                x: rb_ast.x,
                                v: V2::new(
                                    2.0 * rng.gen::<f32>() - 1.0,
                                    2.0 * rng.gen::<f32>() - 1.0,
                                ),
                                phi: rng.gen::<f32>(),
                                omega: rng.gen::<f32>() - 0.5,
                            };
                            let rb2 = RigidBody {
                                x: rb_ast.x,
                                v: (m - rb1.v * a2) * (1.0 / a2),
                                phi: rng.gen::<f32>(),
                                omega: (a * rb_ast.omega - a1 * rb1.omega) / a2,
                            };

                            let mut poly1 = Polygon::random();
                            poly1.scale(a1 / poly1.area());
                            let mut poly2 = Polygon::random();
                            poly2.scale(a2 / poly2.area());

                            let ast1 = entities.create();
                            let ast2 = entities.create();

                            updater.insert(ast1, Asteroid);
                            updater.insert(
                                ast1,
                                Color {
                                    color: graphics::Color::WHITE,
                                },
                            );
                            updater.insert(ast1, rb1);
                            updater.insert(ast1, poly1);

                            updater.insert(ast2, Asteroid);
                            updater.insert(
                                ast2,
                                Color {
                                    color: graphics::Color::WHITE,
                                },
                            );
                            updater.insert(ast2, rb2);
                            updater.insert(ast2, poly2);
                            break;
                        } // spawn new asteroids?
                    } // collision?
                } // for each line segment
            } // for each asteroid
        } // for each bullet
    }
}
