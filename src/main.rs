use image::{self as img};

const WIDTH: u32 = 400;
const HEIGHT: u32 = 400;

enum Node {
    X,
    Y,
    Add(Box<Node>, Box<Node>),
    Sub(Box<Node>, Box<Node>),
    Mul(Box<Node>, Box<Node>),
    Div(Box<Node>, Box<Node>),
    Sin,
    Cos,
    Tan,
    Pow,
    Triple(Box<Node>, Box<Node>, Box<Node>),
    Constant(f32),
}

struct Color {
    r: f32,
    g: f32,
    b: f32,
}

#[allow(unused)]
fn gray_gradient(x: f32, _y: f32) -> Color {
    Color {
        r: x,
        g: x,
        b: x,
    }
}

fn split4(x: f32, y: f32) -> Color {
    // Either way works
    // if x * y >= 0.0 {
    if x * y > 0.0 {
        Color {
            r: x,
            g: y,
            b: 1.0,
        }
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

fn render(function: fn(x: f32, y:f32) -> Color) {
    let mut img = img::ImageBuffer::new(WIDTH, HEIGHT);
    for y in 0..HEIGHT {
        let ny = y as f32 / (HEIGHT as f32) * 2.0 - 1.0;
        for x in 0..WIDTH {
            let nx = x as f32 / (WIDTH as f32) * 2.0 - 1.0;
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

fn render_node(node: Node) {}

fn main() {
    render(split4);
}
