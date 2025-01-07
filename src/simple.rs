use image::{self as img};

const WIDTH: i32 = 400;
const HEIGHT: i32 = 400;

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum CompareKind {
    GreaterThan,
    LessThan,
    GreaterThanEqual,
    LessThanEqual,
    Equal,
    NotEqual,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum FnNode {
    X,
    Y,
    Number(f32),
    Add(Box<FnNode>, Box<FnNode>),
    Sub(Box<FnNode>, Box<FnNode>),
    Mul(Box<FnNode>, Box<FnNode>),
    Div(Box<FnNode>, Box<FnNode>),
    Mod(Box<FnNode>, Box<FnNode>),
    Compare(Box<FnNode>, CompareKind, Box<FnNode>),
    If(Box<FnNode>, Box<FnNode>, Box<FnNode>),
    Triple(Box<FnNode>, Box<FnNode>, Box<FnNode>),
    Random,
    Rule(usize),
    // Pow,
    // Sin,
    // Cos,
    // Tan,
}

#[derive(Debug, Clone)]
struct Color {
    r: f32,
    g: f32,
    b: f32,
}

#[allow(dead_code)]
fn split4(x: f32, y: f32) -> Color {
    // Either way works
    // if x * y >= 0.0 {
    if x * y > 0.0 {
        Color { r: x, g: y, b: 1.0 }
    } else {
        // let rem = x % y;
        let rem = (x + 1e-3) % (y + 1e-3);
        Color {
            r: rem,
            g: rem,
            b: rem,
        }
    }
}

#[allow(dead_code)]
fn render(function: fn(x: f32, y: f32) -> Color) {
    let mut img = img::ImageBuffer::new(WIDTH as u32, HEIGHT as u32);
    for y in 0..HEIGHT {
        let ny = (y as i32 / (HEIGHT as i32)) as f32 * 2.0 - 1.0;
        for x in 0..WIDTH {
            let nx = (x as i32 / (WIDTH as i32)) as f32 * 2.0 - 1.0;
            let color = function(nx, ny);
            img.put_pixel(
                x as u32,
                y as u32,
                img::Rgb([
                    (((color.r + 1.0) / 2.0) * 255.0) as u8,
                    (((color.g + 1.0) / 2.0) * 255.0) as u8,
                    (((color.b + 1.0) / 2.0) * 255.0) as u8,
                ]),
            );
        }
    }
    img.save("output.png").unwrap();
}
