mod node;

use image::{self as img};

const WIDTH: u32 = 400;
const HEIGHT: u32 = 400;

#[derive(Debug, Clone)]
enum CompareKind {
    GreaterThan,
    LessThan,
    GreaterThanEqual,
    LessThanEqual,
    Equal,
    NotEqual,
}

#[derive(Debug, Clone)]
enum Node {
    X,
    Y,
    Number(f32),
    Add(Box<Node>, Box<Node>),
    Sub(Box<Node>, Box<Node>),
    Mul(Box<Node>, Box<Node>),
    Div(Box<Node>, Box<Node>),
    Mod(Box<Node>, Box<Node>),
    Compare(Box<Node>, CompareKind, Box<Node>),
    If(Box<Node>, Box<Node>, Box<Node>),
    Triple(Box<Node>, Box<Node>, Box<Node>),
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

fn render(function: fn(x: f32, y: f32) -> Color) {
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

fn node_eval(node: &Node, x: f32, y: f32) -> Option<Node> {
    match node {
        Node::X => Some(Node::Number(x)),
        Node::Y => Some(Node::Number(y)),
        Node::Number(val) => Some(Node::Number(*val)),
        Node::Add(a, b) => {
            let a = node_eval(a, x, y)?;
            let b = node_eval(b, x, y)?;
            match (a, b) {
                (Node::Number(a), Node::Number(b)) => Some(Node::Number(a + b)),
                _ => None,
            }
        }
        Node::Sub(a, b) => {
            let a = node_eval(a, x, y)?;
            let b = node_eval(b, x, y)?;
            match (a, b) {
                (Node::Number(a), Node::Number(b)) => Some(Node::Number(a - b)),
                _ => None,
            }
        }
        Node::Mul(a, b) => {
            let a = node_eval(a, x, y)?;
            let b = node_eval(b, x, y)?;
            match (a, b) {
                (Node::Number(a), Node::Number(b)) => Some(Node::Number(a * b)),
                _ => None,
            }
        }
        Node::Div(a, b) => {
            let a = node_eval(a, x, y)?;
            let b = node_eval(b, x, y)?;
            match (a, b) {
                (Node::Number(a), Node::Number(b)) => Some(Node::Number(a / b)),
                _ => None,
            }
        }
        Node::Mod(a, b) => {
            let a = node_eval(a, x, y)?;
            let b = node_eval(b, x, y)?;
            match (a, b) {
                (Node::Number(a), Node::Number(b)) => Some(Node::Number(a % b)),
                _ => None,
            }
        }
        Node::Compare(a, ord, b) => {
            let a = node_eval(a, x, y)?;
            let b = node_eval(b, x, y)?;
            match ord {
                CompareKind::GreaterThan => match (a, b) {
                    (Node::Number(a), Node::Number(b)) => {
                        Some(Node::Number(if a > b { 1.0 } else { 0.0 }))
                    }
                    _ => None,
                },
                CompareKind::LessThan => match (a, b) {
                    (Node::Number(a), Node::Number(b)) => {
                        Some(Node::Number(if a < b { 1.0 } else { 0.0 }))
                    }
                    _ => None,
                },
                CompareKind::GreaterThanEqual => match (a, b) {
                    (Node::Number(a), Node::Number(b)) => {
                        Some(Node::Number(if a >= b { 1.0 } else { 0.0 }))
                    }
                    _ => None,
                },
                CompareKind::LessThanEqual => match (a, b) {
                    (Node::Number(a), Node::Number(b)) => {
                        Some(Node::Number(if a <= b { 1.0 } else { 0.0 }))
                    }
                    _ => None,
                },
                CompareKind::Equal => match (a, b) {
                    (Node::Number(a), Node::Number(b)) => {
                        Some(Node::Number(if a == b { 1.0 } else { 0.0 }))
                    }
                    _ => None,
                },
                CompareKind::NotEqual => match (a, b) {
                    (Node::Number(a), Node::Number(b)) => {
                        Some(Node::Number(if a != b { 1.0 } else { 0.0 }))
                    }
                    _ => None,
                },
            }
        }
        Node::If(cond, then, elze) => {
            let cond = node_eval(cond, x, y)?;
            match cond {
                Node::Number(1.0) => node_eval(then, x, y),
                Node::Number(0.0) => node_eval(elze, x, y),
                _ => None,
            }
        }
        Node::Triple(a, b, c) => {
            let a = node_eval(a, x, y)?;
            let b = node_eval(b, x, y)?;
            let c = node_eval(c, x, y)?;
            match (a, b, c) {
                (Node::Number(a), Node::Number(b), Node::Number(c)) => Some(Node::Triple(
                    Box::new(Node::Number(a)),
                    Box::new(Node::Number(b)),
                    Box::new(Node::Number(c)),
                )),
                _ => None,
            }
        }
    }
}

fn expr_eval(expr: &Node, x: f32, y: f32) -> Option<Color> {
    // If Node is a triple call node_eval with that node or return None
    match expr {
        Node::Triple(a, b, c) => {
            let a = match node_eval(a, x, y)? {
                Node::Number(val) => val,
                _ => return None,
            };
            let b = match node_eval(b, x, y)? {
                Node::Number(val) => val,
                _ => return None,
            };
            let c = match node_eval(c, x, y)? {
                Node::Number(val) => val,
                _ => return None,
            };
            Some(Color { r: a, g: b, b: c })
        }
        Node::If(cond, then, elze) => {
            let cond = match node_eval(cond, x, y)? {
                Node::Number(val) => val,
                _ => return None,
            };
            if cond > 0.0 {
                expr_eval(then, x, y)
            } else {
                expr_eval(elze, x, y)
            }
        }
        _ => None,
    }
}

fn node_render(expr: &Node) {
    let mut img = img::ImageBuffer::new(WIDTH, HEIGHT);
    for y in 0..HEIGHT {
        let ny = y as f32 / (HEIGHT as f32) * 2.0 - 1.0;
        for x in 0..WIDTH {
            let nx = x as f32 / (WIDTH as f32) * 2.0 - 1.0;
            let color = expr_eval(expr, nx, ny).unwrap();
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

fn main() {
    // Grayscale node
    let grayscale_node = Node::Triple(Box::new(Node::X), Box::new(Node::Y), Box::new(Node::Y));

    let mistake = Node::If(
        Box::new(Node::Compare(Box::new(Node::Mul(Box::new(Node::X), Box::new(Node::Y))), CompareKind::GreaterThan, Box::new(Node::Number(0.0)))),
        Box::new(Node::Triple(Box::new(Node::X), Box::new(Node::Y), Box::new(Node::Number(1.0)))),
        Box::new(Node::Triple(Box::new(Node::Number(0.0)), Box::new(Node::Number(0.0)), Box::new(Node::Number(0.0)))),
    );

    // Split4 node
    let split4_node = Node::If(
        Box::new(Node::Compare(
            Box::new(Node::Mul(Box::new(Node::X), Box::new(Node::Y))),
            CompareKind::GreaterThan,
            Box::new(Node::Number(0.0)),
        )),
        Box::new(Node::Triple(
            Box::new(Node::X),
            Box::new(Node::Y),
            Box::new(Node::Number(1.0)),
        )),
        Box::new(Node::Triple(
            Box::new(Node::Mod(
                Box::new(Node::Add(Box::new(Node::X), Box::new(Node::Number(1e-3)))),
                Box::new(Node::Add(Box::new(Node::Y), Box::new(Node::Number(1e-3))))
            )),
            Box::new(Node::Mod(
                Box::new(Node::Add(Box::new(Node::X), Box::new(Node::Number(1e-3)))),
                Box::new(Node::Add(Box::new(Node::Y), Box::new(Node::Number(1e-3)))),
            )),
            Box::new(Node::Mod(
                Box::new(Node::Add(Box::new(Node::X), Box::new(Node::Number(1e-3)))),
                Box::new(Node::Add(Box::new(Node::Y), Box::new(Node::Number(1e-3)))),
            )),
        )),
    );

    let node = split4_node;
    node_render(&node);
    // render(split4);
}
