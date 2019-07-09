// Draw some multi-colored geometry to the screen
extern crate quicksilver;
extern crate rand;
extern crate specs;

use quicksilver::prelude::*;
use quicksilver::{geom, graphics, lifecycle};

//type V2 = vector2d::Vector2D<f32>;
type V2 = quicksilver::geom::Vector;

mod components;
use components::*;
mod physics;

/*
 * Quicksilver stuff
 */
#[derive(Debug)]
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
        let mut rng = rand::thread_rng();
        let two_pi: f32 = 2.0 * std::f32::consts::PI;
        let rb = RigidBody {
            x: V2::new(100.0 * rng.gen::<f32>(), 100.0 * rng.gen::<f32>()),
            v: V2::new(
                10.0 * rng.gen::<f32>() - 5.0,
                10.0 * rng.gen::<f32>() - 5.0,
            ),
            phi: two_pi * rng.gen::<f32>(),
            omega: 2.0 * rng.gen::<f32>() - 1.0,
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
            .with(physics::PhysicsUpdate, "physics", &[])
            .with(physics::BulletAsteroidCollision, "ba-collision", &[])
            .build();
        Ok(GameSession {
            state: GameState::Init,
            world: world,
            dispatcher: dispatcher,
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
                let v0 = physics::euclidean(&poly.pts[k], rb.phi, rb.x);
                let v1 = physics::euclidean(&poly.pts[k + 1], rb.phi, rb.x);
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
