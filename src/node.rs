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

impl Node {
    const WIDTH: u32 = 400;
    const HEIGHT: u32 = 400;

    pub fn node_eval(&self, x: f32, y: f32) -> Option<Node> {
        match self {
            Node::X => Some(Node::Number(x)),
            Node::Y => Some(Node::Number(y)),
            Node::Number(val) => Some(Node::Number(*val)),
            Node::Add(a, b) => {
                let a = a.node_eval(x, y)?;
                let b = b.node_eval(x, y)?;
                match (a, b) {
                    (Node::Number(a), Node::Number(b)) => Some(Node::Number(a + b)),
                    _ => None,
                }
            }
            Node::Sub(a, b) => {
                let a = a.node_eval(x, y)?;
                let b = b.node_eval(x, y)?;
                match (a, b) {
                    (Node::Number(a), Node::Number(b)) => Some(Node::Number(a - b)),
                    _ => None,
                }
            }
            Node::Mul(a, b) => {
                let a = a.node_eval(x, y)?;
                let b = b.node_eval(x, y)?;
                match (a, b) {
                    (Node::Number(a), Node::Number(b)) => Some(Node::Number(a * b)),
                    _ => None,
                }
            }
            Node::Div(a, b) => {
                let a = a.node_eval(x, y)?;
                let b = b.node_eval(x, y)?;
                match (a, b) {
                    (Node::Number(a), Node::Number(b)) => Some(Node::Number(a / b)),
                    _ => None,
                }
            }
            Node::Mod(a, b) => {
                let a = a.node_eval(x, y)?;
                let b = b.node_eval(x, y)?;
                match (a, b) {
                    (Node::Number(a), Node::Number(b)) => Some(Node::Number(a % b)),
                    _ => None,
                }
            }
            Node::Compare(a, ord, b) => {
                let a = a.node_eval(x, y)?;
                let b = b.node_eval(x, y)?;
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
                let cond = cond.node_eval(x, y)?;
                match cond {
                    Node::Number(1.0) => then.node_eval(x, y),
                    Node::Number(0.0) => elze.node_eval(x, y),
                    _ => None,
                }
            }
            Node::Triple(a, b, c) => {
                let a = a.node_eval(x, y)?;
                let b = b.node_eval(x, y)?;
                let c = c.node_eval(x, y)?;
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

    fn expr_eval(&self, x: f32, y: f32) -> Option<Color> {
        // If Node is a triple call node_eval with that node or return None
        match self {
            Node::Triple(a, b, c) => {
                let a = match a.node_eval(x, y)? {
                    Node::Number(val) => val,
                    _ => return None,
                };
                let b = match b.node_eval(x, y)? {
                    Node::Number(val) => val,
                    _ => return None,
                };
                let c = match c.node_eval(x, y)? {
                    Node::Number(val) => val,
                    _ => return None,
                };
                Some(Color { r: a, g: b, b: c })
            }
            Node::If(cond, then, elze) => {
                let cond = match cond.node_eval(x, y)? {
                    Node::Number(val) => val,
                    _ => return None,
                };
                if cond > 0.0 {
                    then.expr_eval(x, y)
                } else {
                    elze.expr_eval(x, y)
                }
            }
            _ => None,
        }
    }

    fn node_render(expr: &Node) {
        let h = Node::HEIGHT;
        let w = Node::WIDTH;
        let mut img = img::ImageBuffer::new(w, h);
        for y in 0..h {
            let ny = y as f32 / (h as f32) * 2.0 - 1.0;
            for x in 0..w {
                let nx = x as f32 / (w as f32) * 2.0 - 1.0;
                let color = expr.expr_eval(nx, ny).unwrap();
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
}
