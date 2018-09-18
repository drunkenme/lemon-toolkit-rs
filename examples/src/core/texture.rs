#[macro_use]
extern crate crayon_testbed;
use crayon_testbed::prelude::*;

impl_vertex!{
    Vertex {
        position => [Position; Float; 2; false],
    }
}

struct Window {
    surface: SurfaceHandle,
    shader: ShaderHandle,
    mesh: MeshHandle,
    texture: TextureHandle,
    canvas: ConsoleCanvas,
}

impl Window {
    fn new(engine: &mut Engine) -> Result<Self> {
        let ctx = engine.context();

        let verts: [Vertex; 4] = [
            Vertex::new([-1.0, -1.0]),
            Vertex::new([1.0, -1.0]),
            Vertex::new([1.0, 1.0]),
            Vertex::new([-1.0, 1.0]),
        ];
        let idxes: [u16; 6] = [0, 1, 2, 0, 2, 3];

        // Create vertex buffer object.
        let mut params = MeshParams::default();
        params.num_verts = 4;
        params.num_idxes = 6;
        params.layout = Vertex::layout();

        let data = MeshData {
            vptr: Vertex::encode(&verts[..]).into(),
            iptr: IndexFormat::encode(&idxes).into(),
        };

        let mesh = ctx.video.create_mesh(params, Some(data))?;

        // Create the view state.
        let setup = SurfaceParams::default();
        let surface = ctx.video.create_surface(setup)?;

        // Create shader state.
        let attributes = AttributeLayout::build()
            .with(Attribute::Position, 2)
            .finish();

        let uniforms = UniformVariableLayout::build()
            .with("renderedTexture", UniformVariableType::Texture)
            .finish();

        let mut params = ShaderParams::default();
        params.attributes = attributes;
        params.uniforms = uniforms;
        let vs = include_str!("shaders/texture.vs").to_owned();
        let fs = include_str!("shaders/texture.fs").to_owned();
        let shader = ctx.video.create_shader(params, vs, fs)?;
        let texture = ctx.video.create_texture_from("res:crate.bmp")?;

        Ok(Window {
            surface: surface,
            shader: shader,
            mesh: mesh,
            texture: texture,
            canvas: ConsoleCanvas::new(&ctx, None)?,
        })
    }
}

impl Application for Window {
    fn on_update(&mut self, ctx: &Context) -> Result<()> {
        let mut dc = DrawCall::new(self.shader, self.mesh);
        dc.set_uniform_variable("renderedTexture", self.texture);
        ctx.video.draw(self.surface, dc);

        self.canvas.render(ctx);
        Ok(())
    }

    fn on_post_update(&mut self, _: &Context, info: &FrameInfo) -> Result<()> {
        self.canvas.update(info);
        Ok(())
    }

    fn on_exit(&mut self, ctx: &Context) -> Result<()> {
        ctx.video.delete_mesh(self.mesh);
        ctx.video.delete_shader(self.shader);
        ctx.video.delete_surface(self.surface);
        Ok(())
    }
}

fn main() {
    let res = crayon_testbed::find_res_dir();

    let params = crayon_testbed::settings("CR: Texture", (464, 434));
    let mut engine = Engine::new_with(&params).unwrap();
    engine.res.mount("res", res).unwrap();

    let window = Window::new(&mut engine).unwrap();
    engine.run(window).unwrap();
}
