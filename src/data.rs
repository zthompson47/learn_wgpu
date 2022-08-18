use crate::Vertex;

#[rustfmt::skip]
pub const VERTICES: &[Vertex] = &[
    // Pentagon
    Vertex { position: [-0.0868241, 0.49240386, 0.0], tex_coords: [0.4131759, 1.0 - 0.99240386], },
    Vertex { position: [-0.49513406, 0.06958647, 0.0], tex_coords: [0.0048659444, 1.0 - 0.56958647], },
    Vertex { position: [-0.21918549, -0.44939706, 0.0], tex_coords: [0.28081453, 1.0 - 0.05060294], },
    Vertex { position: [0.35966998, -0.3473291, 0.0], tex_coords: [0.85967, 1.0 - 0.1526709], },
    Vertex { position: [0.44147372, 0.2347359, 0.0], tex_coords: [0.9414737, 1.0 - 0.7347359], },
    // Shape
    Vertex { position: [0.0, 0.0, 0.0], tex_coords: [0.4131759, 0.99240386], },
    Vertex { position: [0.5, 0.2, 0.0], tex_coords: [0.0048659444, 0.56958647], },
    Vertex { position: [0.5, 0.2, 0.0], tex_coords: [0.28081453, 0.05060294], },
    Vertex { position: [0.25, 0.4, 0.0], tex_coords: [0.85967, 0.1526709], },
    Vertex { position: [0.0, 0.6, 0.0], tex_coords: [0.9414737, 0.7347359], },
    Vertex { position: [-0.25, 0.4, 0.0], tex_coords: [0.4131759, 0.99240386], },
    Vertex { position: [-0.5, 0.2, 0.0], tex_coords: [0.0048659444, 0.56958647], },
    Vertex { position: [-0.6, 0.0, 0.0], tex_coords: [0.28081453, 0.05060294], },
    Vertex { position: [-0.5, -0.2, 0.0], tex_coords: [0.85967, 0.1526709], },
    Vertex { position: [-0.25, -0.4, 0.0], tex_coords: [0.9414737, 0.7347359], },
    Vertex { position: [0.0, -0.6, 0.0], tex_coords: [0.4131759, 0.99240386], },
    Vertex { position: [0.25, -0.4, 0.0], tex_coords: [0.0048659444, 0.56958647], },
    Vertex { position: [0.5, -0.2, 0.0], tex_coords: [0.28081453, 0.05060294], },
];

#[rustfmt::skip]
pub const INDICES: &[u16] = &[
    // Pentagon
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
    // Shape
    0, 1, 2,
    0, 2, 3,
    0, 3, 4,
    0, 4, 5,
    0, 5, 6,
    0, 6, 7,
    0, 7, 8,
    0, 8, 9,
    0, 9, 10,
    0, 10, 11,
    0, 11, 12,
];

// Old triangle.
/*const VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.5, 0.0],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.0],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
        color: [0.0, 0.0, 1.0],
    },
];*/
// Old color vertices.
/*
Vertex { position: [-0.0868241, 0.49240386, 0.0], color: [0.5, 0.0, 0.5], },
Vertex { position: [-0.49513406, 0.06958647, 0.0], color: [0.5, 0.0, 0.5], },
Vertex { position: [-0.21918549, -0.44939706, 0.0], color: [0.5, 0.0, 0.5], },
Vertex { position: [0.35966998, -0.3473291, 0.0], color: [0.5, 0.0, 0.5], },
Vertex { position: [0.44147372, 0.2347359, 0.0], color: [0.5, 0.0, 0.5], },
*/
