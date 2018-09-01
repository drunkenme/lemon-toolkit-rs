use crayon::math;

use scene::Transform;

#[derive(Debug, Clone, Copy)]
pub struct Lit {
    /// Is this light enable.
    pub enable: bool,
    /// Is this light casting shadow.
    pub shadow_caster: bool,
    /// Color of the light.
    pub color: math::Color<f32>,
    /// Brightness of the light source, in lumens.
    pub intensity: f32,
    /// Lit source
    pub source: LitSource,

    #[doc(hidden)]
    pub(crate) transform: Transform,
}

/// Enumeration for all light sources.
#[derive(Debug, Clone, Copy)]
pub enum LitSource {
    /// A direcitonal light.
    Dir,
    /// A point light.
    Point {
        /// Maximum raidus of the point light's affected data.
        radius: f32,
        /// Smoothness of the light-to-dark transition from the center to the radius.
        smoothness: f32,
    },
}

impl Default for Lit {
    fn default() -> Self {
        Lit {
            enable: true,
            shadow_caster: false,
            color: math::Color::white(),
            intensity: 1.0,
            source: LitSource::Dir,
            transform: Transform::default(),
        }
    }
}
