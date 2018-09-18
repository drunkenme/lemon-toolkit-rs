extern crate crayon_testbed;
use crayon_testbed::prelude::*;

struct Window {
    world: World<SimpleRenderer>,
    room: Entity,
    canvas: ConsoleCanvas,
}

impl Window {
    fn new(engine: &mut Engine) -> Result<Self> {
        //
        let world_resources = WorldResources::new(engine)?;
        let ctx = engine.context();
        let pipeline = SimpleRenderer::new(&ctx, world_resources.shared())?;
        let mut world = World::new(world_resources.shared(), pipeline);

        //
        let prefab = world.res.create_prefab_from("res:cornell_box.obj")?;
        // ctx.res.wait(prefab)?;
        let room = world.instantiate(prefab).unwrap();

        // Lets give shortBox some red color.
        let mut m = SimpleMaterial::default();
        let e = world.find("cornell_box.obj/shortBox").unwrap();
        m.diffuse = [255, 100, 100, 255].into();
        world.renderer.add(e, m);

        let e = world.find("cornell_box.obj/tallBox").unwrap();
        m.diffuse = [55, 55, 255, 255].into();
        world.renderer.add(e, m);

        //
        let lit = world.create();
        let rotation = math::Euler::new(math::Deg(45.0), math::Deg(0.0), math::Deg(0.0));
        world.renderables.add_lit(lit, Lit::default());
        world.scene.set_rotation(lit, rotation);

        //
        let camera = world.create();
        let params = Camera::ortho(4.8, 3.2, 0.1, 5.0);
        let center = [0.0, 0.0, 0.0];
        world.renderables.add_camera(camera, params);
        world.scene.set_position(camera, [0.0, 2.0, -2.0]);
        world.scene.look_at(camera, center, [0.0, 1.0, 0.0]);

        Ok(Window {
            room: room,
            world: world,
            canvas: ConsoleCanvas::new(&ctx, None)?,
        })
    }
}

impl Application for Window {
    fn on_update(&mut self, ctx: &Context) -> Result<()> {
        self.world.advance();

        if let GesturePan::Move { movement, .. } = ctx.input.finger_pan() {
            let rotation = math::Euler::new(
                math::Deg(movement.y),
                math::Deg(-movement.x),
                math::Deg(0.0),
            );

            self.world.scene.rotate(self.room, rotation);
        }

        self.canvas.render(ctx);
        Ok(())
    }

    fn on_post_update(&mut self, _: &Context, info: &FrameInfo) -> Result<()> {
        self.canvas.update(info);
        Ok(())
    }
}

fn main() {
    let params = crayon_testbed::settings("CR: Prefab", (640, 480));
    let mut engine = Engine::new_with(&params).unwrap();
    let res = crayon_testbed::find_res_dir();
    engine.res.mount("res", res).unwrap();
    engine.input.set_touch_emulation(true);

    let window = Window::new(&mut engine).unwrap();
    engine.run(window).unwrap();
}
