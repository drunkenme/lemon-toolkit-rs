//! A stateless, layered, multithread video system with OpenGL backends.
//!
//! # Overview and Goals
//!
//! The management of video effects has become an important topic and key feature of
//! rendering engines. With the increasing number of effects it is not sufficient anymore
//! to only support them, but also to integrate them into the rendering engine in a clean
//! and extensible way.
//!
//! The goal of this work and simultaneously its main contribution is to design and
//! implement an advanced effects framework. Using this framework it should be easy for
//! further applications to combine several small effects like texture mapping, shading
//! and shadowing in an automated and transparent way and apply them to any 3D model.
//! Additionally, it should be possible to integrate new effects and use the provided
//! framework for rapid prototyping.
//!
//! ### Multi Platform
//!
//! Ideally, crayon should be able to run on macOS, windows and popular mobile-platforms.
//! There still are a huge number of performance and feature limited devices, so this
//! video module will always be limited by lower-end 3D APIs like OpenGL ES2.0.
//!
//! ### Stateless Pipeline
//!
//! Ordinary OpenGL application deals with stateful APIs, which is error-prone. This
//! means whenever you change any state in the API for subsequent draw calls, this state
//! change also affects draw calls submitted at a later point in time. Ideally, submitting
//! a draw call with whatever state we want should not affect any of the other draw calls,
//! even in multi-thread environments.
//!
//! Modern 3D-APIs like [gfx-rs](https://github.com/gfx-rs/gfx), [glium](https://github.com/glium/glium)
//! bundles render state and data into a few, precompiled resource objects which are
//! combined into final render pipeline. We should follow the same philosophy.
//!
//! ### Multi-thread
//!
//! In most cases, dividing OpenGL rendering across multiple threads will not result in
//! any performance improvement due the pipeline nature of OpenGL. What we are about
//! to do is actually exploiting parallelism in resource preparation, and provides a set of
//! multi-thread friendly APIs.
//!
//! The most common solution is by using a double-buffer of commands. This consists of
//! running the renderer backend in a speparate thread, where all draw calls and communication
//! with the OpenGL API are performed. The frontend thread that runs the game logic
//! communicates with the backend renderer via a command double-buffer.
//!
//! ### Layered Rendering
//!
//! Its important to sort video commands (generated by different threads) before submiting
//! them to OpenGL, for the sack of both correctness and performance. For example, to draw
//! transparent objects via blending, we need draw opaque object first, usually from front-to-back,
//! and draw translucents from back-to-front.
//!
//! The idea here is to assign a integer key to a command which is used for sorting. Depending
//! on where those bits are stored in the integer, you can apply different sorting criteria
//! for the same array of commands, as long as you know how the keys were built.
//!
//! # Resource Objects
//!
//! Render state and data, which are combined into final render pipeline, are bundled into a
//! few, precompiled resource objects in video module.
//!
//! All resources types can be created instantly from data in memory, and meshes, textures
//! can also be loaded asynchronously from the filesystem.
//!
//! And the actual resource objects are usually private and opaque, you will get a `Handle`
//! immediately for every resource objects you created instead of some kind of reference.
//! Its the unique identifier for the resource, its type-safe and copyable.
//!
//! When you are done with the created resource objects, its your responsiblity to delete the
//! resource object with `Handle` to avoid leaks.
//!
//! For these things loaded from filesystem, it could be safely shared by the `Location`. We
//! keeps a use-counting internally. It will not be freed really, before all the users deletes
//! its `Handle`.
//!
//! ### Surface Object
//!
//! Surface object plays as the `Layer` role we mentioned above, all the commands we submitted
//! in application code is attached to a specific `Surface`. Commands inside `Surface` are
//! sorted before submitting to underlying OpenGL.
//!
//! Surface object also holds references to render target, and wraps rendering operations to
//! it. Likes clearing, offscreen-rendering, MSAA resolve etc..
//!
//! ```rust
//! use crayon::prelude::*;
//! application::headless().unwrap();
//!
//! // Creates a `SurfaceParams` object.
//! let mut params = SurfaceParams::default();
//! /// Sets the attachments of internal frame-buffer. It consists of multiple color attachments
//! /// and a optional `Depth/DepthStencil` buffer attachment.
//! ///
//! /// If none attachment is assigned, the default framebuffer generated by the system will be
//! /// used.
//! params.set_attachments(&[], None);
//! // Sets the clear flags for this surface and its underlying framebuffer.
//! params.set_clear(Color::white(), 1.0, None);
//!
//! // Creates an surface with `SurfaceParams`.
//! let surface = video::create_surface(params).unwrap();
//! // Deletes the surface object.
//! video::delete_surface(surface);
//! ```
//!
//! ### Shader Object
//!
//! Shader object is introduced to encapsulate all stateful things we need to configurate
//! video pipeline. This would also enable us to easily change the order of draw calls
//! and get rid of redundant state changes.
//!
//! ```rust
//! use crayon::prelude::*;
//! application::headless().unwrap();
//!
//! // Declares the uniform variable layouts.
//! let mut uniforms = UniformVariableLayout::build()
//!     .with("u_ModelViewMatrix", UniformVariableType::Matrix4f)
//!     .with("u_MVPMatrix", UniformVariableType::Matrix4f)
//!     .finish();
//!
//! // Declares the attributes.
//! let attributes = AttributeLayout::build()
//!      .with(Attribute::Position, 3)
//!      .with(Attribute::Normal, 3)
//!      .finish();
//!
//! let mut params = ShaderParams::default();
//! params.attributes = attributes;
//! params.uniforms = uniforms;
//! params.state = RenderState::default();
//!
//! let vs = "..".into();
//! let fs = "..".into();
//!
//! // Create a shader with initial shaders and render state. It encapusulates all the
//! // informations we need to configurate graphics pipeline before real drawing.
//! let shader = video::create_shader(params, vs, fs).unwrap();
//!
//! // Deletes shader object.
//! video::delete_shader(shader);
//! ```
//!
//! ### Texture Object
//!
//! A texture object is a container of one or more images. It can be the source of a texture
//! access from a Shader.
//!
//! ```rust
//! use crayon::prelude::*;
//! application::headless().unwrap();
//!
//! let mut params = TextureParams::default();
//!
//! // Create a texture object with optional data. You can fill it later with `update_texture`.
//! let texture = video::create_texture(params, None).unwrap();
//!
//! // Deletes the texture object.
//! video::delete_texture(texture);
//! ```
//!
//! #### Compressed Texture Format
//!
//! _TODO_: Cube texture.
//! _TODO_: 3D texture.
//!
//! ### Mesh Object
//!
//! ```rust
//! use crayon::prelude::*;
//! application::headless().unwrap();
//!
//! let mut params = MeshParams::default();
//!
//! // Create a mesh object with optional data. You can fill it later with `update_mesh`.
//! let mesh = video::create_mesh(params, None).unwrap();
//!
//! // Deletes the mesh object.
//! video::delete_mesh(mesh);
//! ```
//!
//! # Commands
//!
//! _TODO_: CommandBuffer
//! _TODO_: DrawCommandBuffer

/// Maximum number of attributes in vertex layout.
pub const MAX_VERTEX_ATTRIBUTES: usize = 12;
/// Maximum number of attachments in framebuffer.
pub const MAX_FRAMEBUFFER_ATTACHMENTS: usize = 8;
/// Maximum number of uniform variables in shader.
pub const MAX_UNIFORM_VARIABLES: usize = 32;
/// Maximum number of textures in shader.
pub const MAX_UNIFORM_TEXTURE_SLOTS: usize = 8;

#[macro_use]
pub mod assets;
pub mod command;
pub mod errors;

mod system;

mod backends;

pub mod prelude {
    pub use super::assets::prelude::*;
    pub use super::command::{CommandBuffer, Draw, DrawCommandBuffer};
}

use std::sync::Arc;
use uuid::Uuid;

use math::prelude::Aabb2;
use res::utils::prelude::ResourceState;
use utils::double_buf::DoubleBuf;

use self::assets::prelude::*;
use self::backends::frame::Frame;
use self::errors::*;
use self::ins::{ctx, CTX};
use self::system::VideoSystem;

/// Setup the video system.
pub(crate) unsafe fn setup() -> ::errors::Result<()> {
    debug_assert!(CTX.is_null(), "duplicated setup of video system.");

    let ctx = VideoSystem::new()?;
    CTX = Box::into_raw(Box::new(ctx));
    Ok(())
}

/// Setup the video system.
pub(crate) unsafe fn headless() {
    debug_assert!(CTX.is_null(), "duplicated setup of video system.");

    let ctx = VideoSystem::headless();
    CTX = Box::into_raw(Box::new(ctx));
}

/// Discard the video system.
pub(crate) unsafe fn discard() {
    if CTX.is_null() {
        return;
    }

    drop(Box::from_raw(CTX as *mut VideoSystem));
    CTX = 0 as *const VideoSystem;
}

pub(crate) unsafe fn frames() -> Arc<DoubleBuf<Frame>> {
    ctx().frames()
}

/// Creates an surface with `SurfaceParams`.
#[inline]
pub fn create_surface(params: SurfaceParams) -> Result<SurfaceHandle> {
    ctx().create_surface(params)
}

/// Gets the `SurfaceParams` if available.
#[inline]
pub fn surface(handle: SurfaceHandle) -> Option<SurfaceParams> {
    ctx().surface(handle)
}

/// Get the resource state of specified surface.
#[inline]
pub fn surface_state(handle: SurfaceHandle) -> ResourceState {
    ctx().surface_state(handle)
}

/// Deletes surface object.
#[inline]
pub fn delete_surface(handle: SurfaceHandle) {
    ctx().delete_surface(handle)
}

/// Create a shader with initial shaders and render state. It encapusulates all the
/// informations we need to configurate graphics pipeline before real drawing.
#[inline]
pub fn create_shader(params: ShaderParams, vs: String, fs: String) -> Result<ShaderHandle> {
    ctx().create_shader(params, vs, fs)
}

/// Gets the `ShaderParams` if available.
#[inline]
pub fn shader(handle: ShaderHandle) -> Option<ShaderParams> {
    ctx().shader(handle)
}

/// Get the resource state of specified shader.
#[inline]
pub fn shader_state(handle: ShaderHandle) -> ResourceState {
    ctx().shader_state(handle)
}

/// Delete shader state object.
#[inline]
pub fn delete_shader(handle: ShaderHandle) {
    ctx().delete_shader(handle)
}

/// Create a new mesh object.
#[inline]
pub fn create_mesh<T>(params: MeshParams, data: T) -> ::errors::Result<MeshHandle>
where
    T: Into<Option<MeshData>>,
{
    ctx().create_mesh(params, data)
}

/// Creates a mesh object from file asynchronously.
#[inline]
pub fn create_mesh_from<T: AsRef<str>>(url: T) -> ::errors::Result<MeshHandle> {
    ctx().create_mesh_from(url)
}

/// Creates a mesh object from file asynchronously.
#[inline]
pub fn create_mesh_from_uuid(uuid: Uuid) -> ::errors::Result<MeshHandle> {
    ctx().create_mesh_from_uuid(uuid)
}

/// Gets the `MeshParams` if available.
#[inline]
pub fn mesh(handle: MeshHandle) -> Option<MeshParams> {
    ctx().mesh(handle)
}

/// Get the resource state of specified mesh.
#[inline]
pub fn mesh_state(handle: MeshHandle) -> ResourceState {
    ctx().mesh_state(handle)
}

/// Update a subset of dynamic vertex buffer. Use `offset` specifies the offset
/// into the buffer object's data store where data replacement will begin, measured
/// in bytes.
#[inline]
pub fn update_vertex_buffer(
    handle: MeshHandle,
    offset: usize,
    data: &[u8],
) -> ::errors::Result<()> {
    ctx().update_vertex_buffer(handle, offset, data)
}

/// Update a subset of dynamic index buffer. Use `offset` specifies the offset
/// into the buffer object's data store where data replacement will begin, measured
/// in bytes.
#[inline]
pub fn update_index_buffer(handle: MeshHandle, offset: usize, data: &[u8]) -> ::errors::Result<()> {
    ctx().update_index_buffer(handle, offset, data)
}

/// Delete mesh object.
#[inline]
pub fn delete_mesh(handle: MeshHandle) {
    ctx().delete_mesh(handle);
}

/// Create texture object. A texture is an image loaded in video memory,
/// which can be sampled in shaders.
#[inline]
pub fn create_texture<T>(params: TextureParams, data: T) -> ::errors::Result<TextureHandle>
where
    T: Into<Option<TextureData>>,
{
    ctx().create_texture(params, data)
}

/// Creates a texture object from file asynchronously.
#[inline]
pub fn create_texture_from<T: AsRef<str>>(url: T) -> ::errors::Result<TextureHandle> {
    ctx().create_texture_from(url)
}

/// Creates a texture object from file asynchronously.
#[inline]
pub fn create_texture_from_uuid(uuid: Uuid) -> ::errors::Result<TextureHandle> {
    ctx().create_texture_from_uuid(uuid)
}

/// Get the resource state of specified texture.
#[inline]
pub fn texture_state(handle: TextureHandle) -> ResourceState {
    ctx().texture_state(handle)
}

/// Update a contiguous subregion of an existing two-dimensional texture object.
#[inline]
pub fn update_texture(
    handle: TextureHandle,
    area: Aabb2<u32>,
    data: &[u8],
) -> ::errors::Result<()> {
    ctx().update_texture(handle, area, data)
}

/// Delete the texture object.
#[inline]
pub fn delete_texture(handle: TextureHandle) {
    ctx().delete_texture(handle);
}

/// Create render texture object, which could be attached with a framebuffer.
#[inline]
pub fn create_render_texture(params: RenderTextureParams) -> Result<RenderTextureHandle> {
    ctx().create_render_texture(params)
}

/// Gets the `RenderTextureParams` if available.
#[inline]
pub fn render_texture(handle: RenderTextureHandle) -> Option<RenderTextureParams> {
    ctx().render_texture(handle)
}

/// Get the resource state of specified render texture.
#[inline]
pub fn render_texture_state(handle: RenderTextureHandle) -> ResourceState {
    ctx().render_texture_state(handle)
}

/// Delete the render texture object.
#[inline]
pub fn delete_render_texture(handle: RenderTextureHandle) {
    ctx().delete_render_texture(handle)
}

mod ins {
    use super::system::VideoSystem;

    pub static mut CTX: *const VideoSystem = 0 as *const VideoSystem;

    #[inline]
    pub fn ctx() -> &'static VideoSystem {
        unsafe {
            debug_assert!(
                !CTX.is_null(),
                "video system has not been initialized properly."
            );

            &*CTX
        }
    }
}
