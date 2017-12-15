//! Immutable or dynamic vertex and index data.

use graphics::MAX_VERTEX_ATTRIBUTES;

/// Hint abouts the intended update strategy of the data.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BufferHint {
    /// The resource is initialized with data and cannot be changed later, this
    /// is the most common and most efficient usage. Optimal for render targets
    /// and resourced memory.
    Immutable,
    /// The resource is initialized without data, but will be be updated by the
    /// CPU in each frame.
    Stream,
    /// The resource is initialized without data and will be written by the CPU
    /// before use, updates will be infrequent.
    Dynamic,
}

/// Defines how the input vertex data is used to assemble primitives.
#[derive(Debug, Clone, Copy)]
pub enum Primitive {
    /// Separate points.
    Points,
    /// Separate lines.
    Lines,
    /// Line strips.
    LineStrip,
    /// Separate triangles.
    Triangles,
    /// Triangle strips.
    TriangleStrip,
}

impl Primitive {
    pub fn assemble(&self, indices: u32) -> u32 {
        match *self {
            Primitive::Points => indices,
            Primitive::Lines => indices / 2,
            Primitive::LineStrip => indices - 1,
            Primitive::Triangles => indices / 3,
            Primitive::TriangleStrip => indices - 2,
        }
    }

    pub fn assemble_triangles(&self, indices: u32) -> u32 {
        match *self {
            Primitive::Points => 0,
            Primitive::Lines => 0,
            Primitive::LineStrip => 0,
            Primitive::Triangles => indices / 3,
            Primitive::TriangleStrip => indices - 2,
        }

    }
}


#[derive(Debug, Copy, Clone)]
pub struct IndexBufferSetup {
    /// Usage hints.
    pub hint: BufferHint,
    /// The number of indices in this buffer.
    pub num: u32,
    /// The format.
    pub format: IndexFormat,
}

impl Default for IndexBufferSetup {
    fn default() -> Self {
        IndexBufferSetup {
            hint: BufferHint::Immutable,
            num: 0,
            format: IndexFormat::U16,
        }
    }
}

impl_handle!(IndexBufferHandle);

impl IndexBufferSetup {
    pub fn len(&self) -> usize {
        self.num as usize * self.format.len()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct VertexBufferSetup {
    pub hint: BufferHint,
    pub layout: VertexLayout,
    pub num: u32,
}

impl VertexBufferSetup {
    #[inline]
    pub fn len(&self) -> usize {
        self.num as usize * self.layout.stride() as usize
    }
}

impl Default for VertexBufferSetup {
    fn default() -> Self {
        VertexBufferSetup {
            hint: BufferHint::Immutable,
            layout: VertexLayout::default(),
            num: 0,
        }
    }
}

impl_handle!(VertexBufferHandle);

/// Vertex indices can be either 16- or 32-bit. You should always prefer
/// 16-bit indices over 32-bit indices, since the latter may have performance
/// penalties on some platforms, and they take up twice as much memory.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum IndexFormat {
    U16,
    U32,
}

impl IndexFormat {
    pub fn len(&self) -> usize {
        match self {
            &IndexFormat::U16 => 2,
            &IndexFormat::U32 => 4,
        }
    }

    pub fn as_bytes<T>(values: &[T]) -> &[u8]
        where T: Copy
    {
        let len = values.len() * ::std::mem::size_of::<T>();
        unsafe { ::std::slice::from_raw_parts(values.as_ptr() as *const u8, len) }
    }
}

/// The data type in the vertex component.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VertexFormat {
    Byte,
    UByte,
    Short,
    UShort,
    Float,
}

/// The possible pre-defined and named attributes in the vertex component, describing
/// what the vertex component is used for.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VertexAttribute {
    Position = 0,
    Normal = 1,
    Tangent = 2,
    Bitangent = 3,
    Color0 = 4,
    Color1 = 5,
    Indices = 6,
    Weight = 7,
    Texcoord0 = 8,
    Texcoord1 = 9,
    Texcoord2 = 10,
    Texcoord3 = 11,
}

impl Into<&'static str> for VertexAttribute {
    fn into(self) -> &'static str {
        match self {
            VertexAttribute::Position => "Position",
            VertexAttribute::Normal => "Normal",
            VertexAttribute::Tangent => "Tangent",
            VertexAttribute::Bitangent => "Bitangent",
            VertexAttribute::Color0 => "Color0",
            VertexAttribute::Color1 => "Color1",
            VertexAttribute::Indices => "Indices",
            VertexAttribute::Weight => "Weight",
            VertexAttribute::Texcoord0 => "Texcoord0",
            VertexAttribute::Texcoord1 => "Texcoord1",
            VertexAttribute::Texcoord2 => "Texcoord2",
            VertexAttribute::Texcoord3 => "Texcoord3",
        }
    }
}

impl VertexAttribute {
    pub fn from_str(v: &str) -> Option<VertexAttribute> {
        let attributes = [VertexAttribute::Position,
                          VertexAttribute::Normal,
                          VertexAttribute::Tangent,
                          VertexAttribute::Bitangent,
                          VertexAttribute::Color0,
                          VertexAttribute::Color1,
                          VertexAttribute::Indices,
                          VertexAttribute::Weight,
                          VertexAttribute::Texcoord0,
                          VertexAttribute::Texcoord1,
                          VertexAttribute::Texcoord2,
                          VertexAttribute::Texcoord3];
        for at in &attributes {
            let w: &'static str = (*at).into();
            if v == w {
                return Some(*at);
            }
        }

        None
    }
}

/// The details of a vertex attribute.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct VertexAttributeDesc {
    /// The name of this description.
    pub name: VertexAttribute,
    /// The data type of each component of this element.
    pub format: VertexFormat,
    /// The number of components per generic vertex element.
    pub size: u8,
    /// Whether fixed-point data values should be normalized.
    pub normalized: bool,
}

impl Default for VertexAttributeDesc {
    fn default() -> Self {
        VertexAttributeDesc {
            name: VertexAttribute::Position,
            format: VertexFormat::Byte,
            size: 0,
            normalized: false,
        }
    }
}

/// `VertexLayout` defines how a single vertex structure looks like.  A vertex
/// layout is a collection of vertex components, and each vertex component
/// consists of a vertex attribute and the vertex format.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct VertexLayout {
    stride: u8,
    len: u8,
    offset: [u8; MAX_VERTEX_ATTRIBUTES],
    elements: [VertexAttributeDesc; MAX_VERTEX_ATTRIBUTES],
}

impl VertexLayout {
    /// Creates a new an empty `VertexLayoutBuilder`.
    #[inline]
    pub fn build() -> VertexLayoutBuilder {
        VertexLayoutBuilder::new()
    }

    /// Stride of single vertex structure.
    #[inline]
    pub fn stride(&self) -> u8 {
        self.stride
    }

    /// Returns the number of elements in the layout.
    #[inline]
    pub fn len(&self) -> u8 {
        self.len
    }

    /// Relative element offset from the layout.
    pub fn offset(&self, name: VertexAttribute) -> Option<u8> {
        for i in 0..self.elements.len() {
            match self.elements[i].name {
                v if v == name => return Some(self.offset[i]),
                _ => (),
            }
        }

        None
    }

    /// Returns named `VertexAttribute` from the layout.
    pub fn element(&self, name: VertexAttribute) -> Option<VertexAttributeDesc> {
        for i in 0..self.elements.len() {
            match self.elements[i].name {
                v if v == name => return Some(self.elements[i]),
                _ => (),
            }
        }

        None
    }
}

/// Helper structure to build a vertex layout.
#[derive(Default)]
pub struct VertexLayoutBuilder(VertexLayout);

impl VertexLayoutBuilder {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with(&mut self,
                attribute: VertexAttribute,
                format: VertexFormat,
                size: u8,
                normalized: bool)
                -> &mut Self {
        assert!(size > 0 && size <= 4);

        let desc = VertexAttributeDesc {
            name: attribute,
            format: format,
            size: size,
            normalized: normalized,
        };

        for i in 0..self.0.len {
            let i = i as usize;
            if self.0.elements[i].name == attribute {
                self.0.elements[i] = desc;
                return self;
            }
        }

        assert!((self.0.len as usize) < MAX_VERTEX_ATTRIBUTES);
        self.0.elements[self.0.len as usize] = desc;
        self.0.len += 1;

        self
    }

    #[inline]
    pub fn finish(&mut self) -> VertexLayout {
        self.0.stride = 0;
        for i in 0..self.0.len {
            let i = i as usize;
            let len = self.0.elements[i].size * size_of_vertex(self.0.elements[i].format);
            self.0.offset[i] = self.0.stride;
            self.0.stride += len;
        }
        self.0
    }
}

fn size_of_vertex(format: VertexFormat) -> u8 {
    match format {
        VertexFormat::Byte | VertexFormat::UByte => 1,
        VertexFormat::Short | VertexFormat::UShort => 2,
        VertexFormat::Float => 4,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() {
        let layout = VertexLayout::build()
            .with(VertexAttribute::Position, VertexFormat::Float, 3, true)
            .with(VertexAttribute::Texcoord0, VertexFormat::Float, 2, true)
            .finish();

        assert_eq!(layout.stride(), 20);
        assert_eq!(layout.offset(VertexAttribute::Position), Some(0));
        assert_eq!(layout.offset(VertexAttribute::Texcoord0), Some(12));
        assert_eq!(layout.offset(VertexAttribute::Normal), None);

        let element = layout.element(VertexAttribute::Position).unwrap();
        assert_eq!(element.format, VertexFormat::Float);
        assert_eq!(element.size, 3);
        assert_eq!(element.normalized, true);
        assert_eq!(layout.element(VertexAttribute::Normal), None);
    }

    #[test]
    fn rewrite() {
        let layout = VertexLayout::build()
            .with(VertexAttribute::Position, VertexFormat::Byte, 1, false)
            .with(VertexAttribute::Texcoord0, VertexFormat::Float, 2, true)
            .with(VertexAttribute::Position, VertexFormat::Float, 3, true)
            .finish();

        assert_eq!(layout.stride(), 20);
        assert_eq!(layout.offset(VertexAttribute::Position), Some(0));
        assert_eq!(layout.offset(VertexAttribute::Texcoord0), Some(12));
        assert_eq!(layout.offset(VertexAttribute::Normal), None);

        let element = layout.element(VertexAttribute::Position).unwrap();
        assert_eq!(element.format, VertexFormat::Float);
        assert_eq!(element.size, 3);
        assert_eq!(element.normalized, true);
        assert_eq!(layout.element(VertexAttribute::Normal), None);
    }
}

#[macro_use]
pub mod macros {
    use super::*;

    #[doc(hidden)]
    #[derive(Default)]
    pub struct CustomVertexLayoutBuilder(VertexLayout);

    impl CustomVertexLayoutBuilder {
        #[inline]
        pub fn new() -> Self {
            Default::default()
        }

        pub fn with(&mut self,
                    attribute: VertexAttribute,
                    format: VertexFormat,
                    size: u8,
                    normalized: bool,
                    offset_of_field: u8)
                    -> &mut Self {
            assert!(size > 0 && size <= 4);

            let desc = VertexAttributeDesc {
                name: attribute,
                format: format,
                size: size,
                normalized: normalized,
            };

            for i in 0..self.0.len {
                let i = i as usize;
                if self.0.elements[i].name == attribute {
                    self.0.elements[i] = desc;
                    return self;
                }
            }

            assert!((self.0.len as usize) < MAX_VERTEX_ATTRIBUTES);
            self.0.offset[self.0.len as usize] = offset_of_field;
            self.0.elements[self.0.len as usize] = desc;
            self.0.len += 1;

            self
        }

        #[inline]
        pub fn finish(&mut self, stride: u8) -> VertexLayout {
            self.0.stride = stride;
            self.0
        }
    }

    #[macro_export]
    macro_rules! offset_of {
        ($ty:ty, $field:ident) => {
            unsafe { &(*(0 as *const $ty)).$field as *const _ as usize }
        }
    }

    #[macro_export]
    macro_rules! impl_vertex {
        ($name: ident { $($field: ident => [$attribute: tt; $format: tt; $size: tt; $normalized: tt],)* }) => (
            #[repr(C)]
            #[derive(Debug, Copy, Clone)]
            pub struct $name {
                $($field: impl_vertex_field!{VertexFormat::$format, $size}, )*
            }

            impl $name {
                pub fn new($($field: impl_vertex_field!{VertexFormat::$format, $size}, ) *) -> Self {
                    $name {
                        $($field: $field,)*
                    }
                }

                pub fn layout() -> $crate::graphics::assets::mesh::VertexLayout {
                    let mut builder = $crate::graphics::assets::mesh::macros::CustomVertexLayoutBuilder::new();

                    $( builder.with(
                        $crate::graphics::assets::mesh::VertexAttribute::$attribute,
                        $crate::graphics::assets::mesh::VertexFormat::$format,
                        $size,
                        $normalized,
                        offset_of!($name, $field) as u8); ) *

                    builder.finish(::std::mem::size_of::<$name>() as u8)
                }

                pub fn as_bytes(values: &[Self]) -> &[u8] {
                    let len = values.len() * ::std::mem::size_of::<Self>();
                    unsafe { ::std::slice::from_raw_parts(values.as_ptr() as *const u8, len) }
                }
            }
        )
    }

    #[macro_export]
    macro_rules! impl_vertex_field {
        (VertexFormat::Byte, 2) => ([i8; 2]);
        (VertexFormat::Byte, 3) => ([i8; 3]);
        (VertexFormat::Byte, 4) => ([i8; 4]);
        (VertexFormat::UByte, 2) => ([u8; 2]);
        (VertexFormat::UByte, 3) => ([u8; 3]);
        (VertexFormat::UByte, 4) => ([u8; 4]);
        (VertexFormat::Short, 2) => ([i16; 2]);
        (VertexFormat::Short, 3) => ([i16; 3]);
        (VertexFormat::Short, 4) => ([i16; 4]);
        (VertexFormat::UShort, 2) => ([u16; 2]);
        (VertexFormat::UShort, 3) => ([u16; 3]);
        (VertexFormat::UShort, 4) => ([u16; 4]);
        (VertexFormat::Float, 2) => ([f32; 2]);
        (VertexFormat::Float, 3) => ([f32; 3]);
        (VertexFormat::Float, 4) => ([f32; 4]);
    }

    #[cfg(test)]
    mod test {
        use super::super::*;

        impl_vertex! {
            Vertex {
                position => [Position; Float; 3; false],
                texcoord => [Texcoord0; Float; 2; false],
            }
        }

        impl_vertex! {
            Vertex2 {
                position => [Position; Float; 2; false],
                color => [Color0; UByte; 4; true],
                texcoord => [Texcoord0; Byte; 2; false],
            }
        }

        fn as_bytes<T>(values: &[T]) -> &[u8]
            where T: Copy
        {
            let len = values.len() * ::std::mem::size_of::<T>();
            unsafe { ::std::slice::from_raw_parts(values.as_ptr() as *const u8, len) }
        }

        #[test]
        fn basic() {
            let layout = Vertex::layout();
            assert_eq!(layout.stride(), 20);
            assert_eq!(layout.offset(VertexAttribute::Position), Some(0));
            assert_eq!(layout.offset(VertexAttribute::Texcoord0), Some(12));
            assert_eq!(layout.offset(VertexAttribute::Normal), None);

            let bytes: [f32; 5] = [1.0, 1.0, 1.0, 0.0, 0.0];
            let bytes = as_bytes(&bytes);
            assert_eq!(bytes,
                       Vertex::as_bytes(&[Vertex::new([1.0, 1.0, 1.0], [0.0, 0.0])]));

            let bytes: [f32; 10] = [1.0, 1.0, 1.0, 0.0, 0.0, 2.0, 2.0, 2.0, 3.0, 3.0];
            let bytes = as_bytes(&bytes);
            assert_eq!(bytes,
                       Vertex::as_bytes(&[Vertex::new([1.0, 1.0, 1.0], [0.0, 0.0]),
                                          Vertex::new([2.0, 2.0, 2.0], [3.0, 3.0])]));
        }

        #[test]
        fn representation() {
            let layout = Vertex::layout();
            assert_eq!(layout.stride() as usize, ::std::mem::size_of::<Vertex>());

            let layout = Vertex2::layout();
            let _v = Vertex2::new([1.0, 1.0], [0, 0, 0, 0], [0, 0]);
            let _b = Vertex2::as_bytes(&[]);
            assert_eq!(layout.stride() as usize, ::std::mem::size_of::<Vertex2>());
        }
    }
}