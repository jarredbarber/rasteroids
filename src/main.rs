// Draw some multi-colored geometry to the screen
extern crate quicksilver;
extern crate rand;
extern crate specs;
#[macro_use]
extern crate specs_derive;

use specs::Component;
use specs::DenseVecStorage;

use quicksilver::prelude::*;
use quicksilver::{
    geom,
    graphics,
    lifecycle
};

/*
 * Components
 */
#[derive(Debug, Component)]
struct Player {
    score: u32,
    health: u32,
    // last_bullet: u32,
}

#[derive(Debug, Copy, Clone, Component)]
struct RigidBody {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    phi: f32,
    omega: f32
}

#[derive(Debug, Component)]
struct Rectangle {
    w: f32,
    h: f32
}

#[derive(Debug, Component)]
struct Polygon {
    x: Vec<f32>,
    y: Vec<f32>,
}

impl Polygon {
    fn new(x: Vec<f32>, y: Vec<f32>) -> Polygon {
        // find the polygon's barycenter
        let mut A: f32 = 0.0;
        for k in 0..(x.len() - 1) {
            A += 0.5*(x[k]*y[k+1] - x[k+1]*y[k]);
        }

        let mut Cx: f32 = 0.0;
        let mut Cy: f32 = 0.0;

        let scale: f32 = 6.0*A;

        for k in 0..(x.len() - 1) {
            let b = (x[k]*y[k+1] - x[k+1]*y[k])/scale;
            Cx += b*(x[k] + x[k+1]);
            Cy += b*(y[k] + y[k+1]);
        }

        Polygon { x: x.iter().map(|&a|  (a - Cx)).collect(),
                  y: y.iter().map(|&a|  (a - Cy)).collect() }
    }
}

#[derive(Debug, Component)]
struct Color { color: graphics::Color }
    

/*
 * Systems
 */

// Systems
struct PhysicsUpdate;
struct Collision;

// Resources

impl<'a> specs::System<'a> for PhysicsUpdate {
    type SystemData = specs::WriteStorage<'a, RigidBody>;
    fn run(&mut self, mut state: Self::SystemData) {
        use specs::Join;
        let dt = 0.06;
        
        for state in (&mut state).join() {
            state.x += dt*state.vx;
            if state.x < 0.0 {
                state.x += 100.0;
            }
            if state.x > 100.0 {
                state.x = 100.0 - state.x;
            }
            state.y += dt*state.vy;
            if state.y < 0.0 {
                state.y += 100.0;
            }
            if state.y > 100.0 {
                state.y = 100.0 - state.y;
            }
            state.phi += dt*state.omega;
        }
    }
}

/*
 * Quicksilver stuff
 */
enum GameState {
    Init,
    Playing,
    GameOver
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
        self.world.create_entity()
            .with(Player{score: 0, health: 100})
            .with(RigidBody{x: 50.0, y: 50.0, vx: 0.0, vy: 0.0, phi: 0.0, omega: 0.0})
            // .with(Rectangle{h: 5.0, w: 5.0})
            .with(Polygon::new(vec![0.0, 5.0, 0.0, 1.25],
                               vec![1.5, 0.0, -1.5, 0.0]))
            .with(Color{color: graphics::Color::WHITE})
            .build();
    }

    fn spawn_asteroid(&mut self) {
        use crate::rand::Rng;
        let two_pi:f32 = 2.0*std::f32::consts::PI;
        let rb = RigidBody {
            x: 100.0*self.rng.gen::<f32>(),
            y: 100.0*self.rng.gen::<f32>(),
            vx: 10.0*self.rng.gen::<f32>() - 5.0,
            vy: 10.0*self.rng.gen::<f32>() - 5.0,
            phi: two_pi*self.rng.gen::<f32>(),
            omega: 2.0*self.rng.gen::<f32>() - 1.0,
        };

        let mut x = Vec::<f32>::new();
        let mut y = Vec::<f32>::new();
        
        let n:usize = self.rng.gen_range(3usize, 12usize);

        for k in 0..n {
            let phi = (k as f32)*two_pi / ((n+1) as f32);
            let r = 1.0 + 14.0*self.rng.gen::<f32>();
            x.push(r*phi.cos());
            y.push(r*phi.sin());
        }

        use specs::Builder;
        self.world.create_entity()
            .with(rb)
            .with(Polygon::new(x, y))
            .with(Color {color: graphics::Color::WHITE})
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

            for (rb, ply) in (&mut pos_storage, &player_storage).join() {
                rb.vx += rb.phi.cos()*dv;
                rb.vy += rb.phi.sin()*dv;
                rb.phi += dphi;

                let r = (rb.vx*rb.vx + rb.vy*rb.vy).sqrt();
                if r > 10.0 {
                    rb.vx *= 10.0/r;
                    rb.vy *= 10.0/r;
                }
            }
        }
    }

    fn spawn_bullet(&mut self) {
        use specs::Builder;
        let mut player_rb:Option<RigidBody> = None;

        {
            use specs::Join;
            let mut pos_storage = self.world.write_storage::<RigidBody>();
            let player_storage = self.world.read_storage::<Player>();

            for (rb, _ply) in (&mut pos_storage, &player_storage).join() {
                // Recoil
                rb.vx -= 0.1*rb.phi.cos();
                rb.vy -= 0.1*rb.phi.sin();

                player_rb = Some(*rb);
            }
        }

        match player_rb {
            Some(rb) => {
                let vb:f32 = 5.0;
                self.world.create_entity()
                    .with(RigidBody{x: rb.x, y: rb.y,
                                    vx: rb.vx + vb*rb.phi.cos(),
                                    vy: rb.vy + vb*rb.phi.sin(), 
                                    phi: rb.phi, omega: 120.0})
                    .with(Rectangle{h: 0.50, w: 1.0})
                    .with(Color {color: graphics::Color::WHITE})
                    .build();
            },
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

        let dispatcher = specs::DispatcherBuilder::new()
            .with(PhysicsUpdate, "physics", &[])
            .build();
        Ok(GameSession { state: GameState::Init, 
                         world: world, 
                         dispatcher: dispatcher,
                         rng: rand::thread_rng(), })
    }


    fn update(&mut self, window: &mut lifecycle::Window) -> quicksilver::Result<()> {
        match self.state {
            GameState::Init => {
                self.state = GameState::Playing;
                self.spawn_player();
            },
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
            },
            GameState::GameOver => ()
        };
        self.dispatcher.dispatch(&mut self.world.res);
        Ok(())
    }  

    fn draw(&mut self, window: &mut lifecycle::Window) -> quicksilver::Result<()> {
        window.clear(graphics::Color::BLACK)?;
        use specs::Join;
        // Need to manually run the render system here
        let pos_storage = self.world.read_storage::<RigidBody>();
        let rect_storage = self.world.read_storage::<Rectangle>();
        let play_storage = self.world.read_storage::<Player>();
        let color_storage = self.world.read_storage::<Color>();
        let poly_storage = self.world.read_storage::<Polygon>();

        for (color, rb, rect) in (&color_storage, &pos_storage, &rect_storage).join() {
            window.draw_ex(&geom::Rectangle::new((rb.x, rb.y), (rect.w, rect.h)),
                        graphics::Background::Col(color.color),
                        geom::Transform::rotate(rb.phi.to_degrees()),
                        10);
        }

        for (color, rb, poly) in (&color_storage, &pos_storage, &poly_storage).join() {
            let n:usize = poly.x.len();
            let c = rb.phi.cos();
            let s = rb.phi.sin();
            for k in 0..n {
                let v0 = quicksilver::geom::Vector::new(rb.x + c*poly.x[k] - s*poly.y[k], 
                                                        rb.y + s*poly.x[k] + c*poly.y[k]);
                let v1 = quicksilver::geom::Vector::new(rb.x 
                                                        + c*poly.x[(k+1).wrapping_rem(n)]
                                                        - s*poly.y[(k+1).wrapping_rem(n)],
                                                        rb.y 
                                                        + c*poly.y[(k+1).wrapping_rem(n)]
                                                        + s*poly.x[(k+1).wrapping_rem(n)]);
                window.draw(&geom::Line::new(v0, v1).with_thickness(0.1),
                            graphics::Background::Col(color.color));
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
