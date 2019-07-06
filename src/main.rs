// Draw some multi-colored geometry to the screen
extern crate quicksilver;
extern crate specs;
#[macro_use]
extern crate specs_derive;

use quicksilver::{
    geom,
    graphics,
    lifecycle
};

/*
 * Components
 */
#[derive(Debug)]
struct RigidBody {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    phi: f32,
    omega: f32
}

#[derive(Debug)]
struct Rectangle {
    w: f32,
    h: f32
}

impl specs::Component for RigidBody {
    type Storage = specs::VecStorage<Self>;
}

impl specs::Component for Rectangle {
    type Storage = specs::VecStorage<Self>;
}

/*
 * Systems
 */

// Systems
struct PhysicsUpdate;
struct Collision;
struct Render;

// Resources

impl<'a> specs::System<'a> for PhysicsUpdate {
    type SystemData = specs::WriteStorage<'a, RigidBody>;
    fn run(&mut self, mut state: Self::SystemData) {
        use specs::Join;
        let dt = 0.06;
        
        for state in (&mut state).join() {
            state.x += dt*state.vx;
            state.y += dt*state.vy;
            state.phi += dt*state.omega;
        }
    }
}

/*
 * Quicksilver stuff
 */
struct GameSession<'a, 'b> {
    world: specs::World,
    dispatcher: specs::Dispatcher<'a, 'b>
}

impl lifecycle::State for GameSession<'static, 'static> {
    fn new() -> quicksilver::Result<Self> {
        use specs::Builder;
        let mut world = specs::World::new();
        world.register::<RigidBody>();
        world.register::<Rectangle>();
        world.create_entity()
            .with(RigidBody{x: 0.0, y: 0.0, vx: 1.0, vy: 0.0, phi: 0.0, omega: 8.0})
            .with(Rectangle{h: 20.0, w: 32.0})
            .build();

        let dispatcher = specs::DispatcherBuilder::new()
            .with(PhysicsUpdate, "physics", &[])
            .build();
        Ok(GameSession { world: world, dispatcher: dispatcher })
    }

    fn update(&mut self, window: &mut lifecycle::Window) -> quicksilver::Result<()> {
        self.dispatcher.dispatch(&mut self.world.res);
        Ok(())
    }   

    fn draw(&mut self, window: &mut lifecycle::Window) -> quicksilver::Result<()> {
        window.clear(graphics::Color::BLACK)?;
        use specs::Join;
        // Need to manually run the render system here
        let pos_storage = self.world.read_storage::<RigidBody>();
        let rect_storage = self.world.read_storage::<Rectangle>();

        for (rb, rect) in (&pos_storage, &rect_storage).join() {
            window.draw_ex(&geom::Rectangle::new((rb.x, rb.y), (rect.w, rect.h)),
                        graphics::Background::Col(graphics::Color::BLUE),
                        geom::Transform::rotate(rb.phi),
                        10);
        }
        
        Ok(())
    }
}

fn main() {
    let mut settings = lifecycle::Settings::default();
    settings.fullscreen = false;
    settings.vsync = false;
    println!("Settings = {:?}", settings);
    lifecycle::run::<GameSession>("rustlike", geom::Vector::new(100, 100), settings);
}
