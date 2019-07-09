// Draw some multi-colored geometry to the screen
extern crate quicksilver;
extern crate rand;
extern crate specs;
#[macro_use]
extern crate specs_derive;

use specs::Component;
use specs::DenseVecStorage;

use quicksilver::prelude::*;
use quicksilver::{geom, graphics, lifecycle};

//type V2 = vector2d::Vector2D<f32>;
type V2 = quicksilver::geom::Vector;

fn rotate(v: &V2, phi: f32) -> V2 {
    let c = phi.cos();
    let s = phi.sin();
    V2::new(c * v.x - s * v.y, s * v.x + c * v.y)
}

/*
 * Components
 */
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

/*
 * Systems
 */

struct PhysicsUpdate;
struct BulletAsteroidCollision;

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
                    let x0 = rb_ast.x + rotate(&ast_poly.pts[k + 0], rb_ast.phi);
                    let x1 = rb_ast.x + rotate(&ast_poly.pts[k + 1], rb_ast.phi);

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
/*
 * Quicksilver stuff
 */
enum GameState {
    Init,
    Playing,
    GameOver,
}

#[derive(Debug)]
enum Command {
    RotLeft,
    RotRight,
    Fwd,
    Bwd,
}

struct GameSession<'a, 'b> {
    state: GameState,
    world: specs::World,
    dispatcher: specs::Dispatcher<'a, 'b>,
    rng: rand::rngs::ThreadRng,
}

impl GameSession<'static, 'static> {
    fn spawn_player(&mut self) {
        use specs::Builder;
        self.world
            .create_entity()
            .with(Player {
                score: 0,
                health: 100,
            })
            .with(RigidBody {
                x: V2::new(50.0, 50.0),
                v: V2::new(0.0, 0.0),
                phi: 0.0,
                omega: 0.0,
            })
            // .with(Rectangle{h: 5.0, w: 5.0})
            .with(Polygon::new(
                vec![0.0, 5.0, 0.0, 1.25],
                vec![1.5, 0.0, -1.5, 0.0],
            ))
            .with(Color {
                color: graphics::Color::BLUE,
            })
            .build();
    }

    fn spawn_asteroid(&mut self) {
        use crate::rand::Rng;
        let two_pi: f32 = 2.0 * std::f32::consts::PI;
        let rb = RigidBody {
            x: V2::new(100.0 * self.rng.gen::<f32>(), 100.0 * self.rng.gen::<f32>()),
            v: V2::new(
                10.0 * self.rng.gen::<f32>() - 5.0,
                10.0 * self.rng.gen::<f32>() - 5.0,
            ),
            phi: two_pi * self.rng.gen::<f32>(),
            omega: 2.0 * self.rng.gen::<f32>() - 1.0,
        };

        use specs::Builder;
        self.world
            .create_entity()
            .with(rb)
            .with(Asteroid)
            .with(Polygon::random())
            .with(Color {
                color: graphics::Color::WHITE,
            })
            .build();
    }

    fn move_player(&mut self, cmd: Command) {
        let rot = 0.1;
        let (dv, dphi) = match cmd {
            Command::Fwd => (1.0, 0.0),
            Command::Bwd => (-1.0, 0.0),
            Command::RotLeft => (0.0, -rot),
            Command::RotRight => (0.0, rot),
        };

        {
            use specs::Join;
            let mut pos_storage = self.world.write_storage::<RigidBody>();
            let player_storage = self.world.read_storage::<Player>();

            for (rb, _ply) in (&mut pos_storage, &player_storage).join() {
                rb.v.x += rb.phi.cos() * dv;
                rb.v.y += rb.phi.sin() * dv;
                rb.phi += dphi;

                let r = rb.v.len();
                if r > 10.0 {
                    rb.v *= 10.0 / r;
                }
            }
        }
    }

    fn spawn_bullet(&mut self) {
        use specs::Builder;
        let mut player_rb: Option<RigidBody> = None;

        {
            use specs::Join;
            let mut pos_storage = self.world.write_storage::<RigidBody>();
            let player_storage = self.world.read_storage::<Player>();

            for (rb, _ply) in (&mut pos_storage, &player_storage).join() {
                // Recoil
                rb.v.x -= 0.1 * rb.phi.cos();
                rb.v.y -= 0.1 * rb.phi.sin();

                player_rb = Some(*rb);
            }
        }

        match player_rb {
            Some(rb) => {
                use rand::Rng;
                let vb: f32 = 15.0;
                let u = V2::new(vb * rb.phi.cos(), vb * rb.phi.sin());
                self.world
                    .create_entity()
                    .with(Bullet)
                    .with(RigidBody {
                        x: rb.x,
                        v: rb.v + u,
                        phi: rb.phi,
                        omega: 6.0 * rand::thread_rng().gen::<f32>(),
                    })
                    .with(Rectangle { h: 0.50, w: 1.0 })
                    .with(Color {
                        color: graphics::Color::RED,
                    })
                    .build();
            }
            None => {}
        };
    }
}

impl lifecycle::State for GameSession<'static, 'static> {
    fn new() -> quicksilver::Result<Self> {
        // use specs::Builder;
        let mut world = specs::World::new();
        world.register::<RigidBody>();
        world.register::<Rectangle>();
        world.register::<Polygon>();
        world.register::<Player>();
        world.register::<Color>();
        world.register::<Bullet>();
        world.register::<Asteroid>();

        let dispatcher = specs::DispatcherBuilder::new()
            .with(PhysicsUpdate, "physics", &[])
            .with(BulletAsteroidCollision, "ba-collision", &[])
            .build();
        Ok(GameSession {
            state: GameState::Init,
            world: world,
            dispatcher: dispatcher,
            rng: rand::thread_rng(),
        })
    }

    fn update(&mut self, window: &mut lifecycle::Window) -> quicksilver::Result<()> {
        match self.state {
            GameState::Init => {
                self.state = GameState::Playing;
                self.spawn_player();
            }
            GameState::Playing => {
                // Process inputs
                if window.keyboard()[Key::Left].is_down() {
                    self.move_player(Command::RotLeft);
                }
                if window.keyboard()[Key::Right].is_down() {
                    self.move_player(Command::RotRight);
                }
                if window.keyboard()[Key::Up].is_down() {
                    self.move_player(Command::Fwd);
                }
                if window.keyboard()[Key::Down].is_down() {
                    self.move_player(Command::Bwd);
                }
                if window.keyboard()[Key::A] == ButtonState::Pressed {
                    self.spawn_asteroid();
                }
                if window.keyboard()[Key::Space] == ButtonState::Pressed {
                    self.spawn_bullet();
                }
            }
            GameState::GameOver => (),
        };
        self.dispatcher.dispatch(&mut self.world.res);
        self.world.maintain();
        Ok(())
    }

    fn draw(&mut self, window: &mut lifecycle::Window) -> quicksilver::Result<()> {
        window.clear(graphics::Color::BLACK)?;
        use specs::Join;
        // Need to manually run the render system here
        let pos_storage = self.world.read_storage::<RigidBody>();
        let rect_storage = self.world.read_storage::<Rectangle>();
        let color_storage = self.world.read_storage::<Color>();
        let poly_storage = self.world.read_storage::<Polygon>();

        for (color, rb, rect) in (&color_storage, &pos_storage, &rect_storage).join() {
            window.draw_ex(
                &geom::Rectangle::new((rb.x.x, rb.x.y), (rect.w, rect.h)),
                graphics::Background::Col(color.color),
                geom::Transform::rotate(rb.phi.to_degrees()),
                10,
            );
        }

        for (color, rb, poly) in (&color_storage, &pos_storage, &poly_storage).join() {
            for k in 0..poly.len() {
                let v0 = rb.x + rotate(&poly.pts[k], rb.phi);
                let v1 = rb.x + rotate(&poly.pts[k + 1], rb.phi);
                window.draw(
                    &geom::Line::new(v0, v1).with_thickness(0.1),
                    graphics::Background::Col(color.color),
                );
            }
        }

        Ok(())
    }
}

fn main() {
    let mut settings = lifecycle::Settings::default();
    settings.fullscreen = false;
    settings.vsync = false;
    lifecycle::run::<GameSession>("rustlike", geom::Vector::new(100, 100), settings);
}
