use image::{self as img};

const WIDTH: u32 = 400;
const HEIGHT: u32 = 400;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum CompareKind {
    GreaterThan,
    LessThan,
    GreaterThanEqual,
    LessThanEqual,
    Equal,
    NotEqual,
}

#[derive(Debug, Clone)]
pub enum FnNode {
    X,
    Y,
    Random,

    Number(f32),
    Rule(usize),

    Add(Box<FnNode>, Box<FnNode>),
    Sub(Box<FnNode>, Box<FnNode>),
    Mul(Box<FnNode>, Box<FnNode>),
    Div(Box<FnNode>, Box<FnNode>),
    Mod(Box<FnNode>, Box<FnNode>),
    Compare(Box<FnNode>, CompareKind, Box<FnNode>),

    If(Box<FnNode>, Box<FnNode>, Box<FnNode>),
    Triple(Box<FnNode>, Box<FnNode>, Box<FnNode>),
    // Pow,
    // Sin,
    // Cos,
    // Tan,
}

impl FnNode {
    pub fn is_terminal(&self) -> bool {
        match self {
            FnNode::Rule(_) => false,
            _ => true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Color {
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

impl FnNode {
    const WIDTH: u32 = 400;
    const HEIGHT: u32 = 400;

    pub fn node_eval(&self, x: f32, y: f32) -> Option<FnNode> {
        match self {
            FnNode::X => Some(FnNode::Number(x)),
            FnNode::Y => Some(FnNode::Number(y)),
            FnNode::Number(val) => Some(FnNode::Number(*val)),
            FnNode::Add(a, b) => {
                let a = a.node_eval(x, y)?;
                let b = b.node_eval(x, y)?;
                match (a, b) {
                    (FnNode::Number(a), FnNode::Number(b)) => Some(FnNode::Number(a + b)),
                    _ => None,
                }
            }
            FnNode::Sub(a, b) => {
                let a = a.node_eval(x, y)?;
                let b = b.node_eval(x, y)?;
                match (a, b) {
                    (FnNode::Number(a), FnNode::Number(b)) => Some(FnNode::Number(a - b)),
                    _ => None,
                }
            }
            FnNode::Mul(a, b) => {
                let a = a.node_eval(x, y)?;
                let b = b.node_eval(x, y)?;
                match (a, b) {
                    (FnNode::Number(a), FnNode::Number(b)) => Some(FnNode::Number(a * b)),
                    _ => None,
                }
            }
            FnNode::Div(a, b) => {
                let a = a.node_eval(x, y)?;
                let b = b.node_eval(x, y)?;
                match (a, b) {
                    (FnNode::Number(a), FnNode::Number(b)) => Some(FnNode::Number(a / b)),
                    _ => None,
                }
            }
            FnNode::Mod(a, b) => {
                let a = a.node_eval(x, y)?;
                let b = b.node_eval(x, y)?;
                match (a, b) {
                    (FnNode::Number(a), FnNode::Number(b)) => Some(FnNode::Number(a % b)),
                    _ => None,
                }
            }
            FnNode::Compare(a, ord, b) => {
                let a = a.node_eval(x, y)?;
                let b = b.node_eval(x, y)?;
                match ord {
                    CompareKind::GreaterThan => match (a, b) {
                        (FnNode::Number(a), FnNode::Number(b)) => {
                            Some(FnNode::Number(if a > b { 1.0 } else { 0.0 }))
                        }
                        _ => None,
                    },
                    CompareKind::LessThan => match (a, b) {
                        (FnNode::Number(a), FnNode::Number(b)) => {
                            Some(FnNode::Number(if a < b { 1.0 } else { 0.0 }))
                        }
                        _ => None,
                    },
                    CompareKind::GreaterThanEqual => match (a, b) {
                        (FnNode::Number(a), FnNode::Number(b)) => {
                            Some(FnNode::Number(if a >= b { 1.0 } else { 0.0 }))
                        }
                        _ => None,
                    },
                    CompareKind::LessThanEqual => match (a, b) {
                        (FnNode::Number(a), FnNode::Number(b)) => {
                            Some(FnNode::Number(if a <= b { 1.0 } else { 0.0 }))
                        }
                        _ => None,
                    },
                    CompareKind::Equal => match (a, b) {
                        (FnNode::Number(a), FnNode::Number(b)) => {
                            Some(FnNode::Number(if a == b { 1.0 } else { 0.0 }))
                        }
                        _ => None,
                    },
                    CompareKind::NotEqual => match (a, b) {
                        (FnNode::Number(a), FnNode::Number(b)) => {
                            Some(FnNode::Number(if a != b { 1.0 } else { 0.0 }))
                        }
                        _ => None,
                    },
                }
            }
            FnNode::If(cond, then, elze) => {
                let cond = cond.node_eval(x, y)?;
                match cond {
                    FnNode::Number(1.0) => then.node_eval(x, y),
                    FnNode::Number(0.0) => elze.node_eval(x, y),
                    _ => None,
                }
            }
            FnNode::Triple(a, b, c) => {
                let a = a.node_eval(x, y)?;
                let b = b.node_eval(x, y)?;
                let c = c.node_eval(x, y)?;
                match (a, b, c) {
                    (FnNode::Number(a), FnNode::Number(b), FnNode::Number(c)) => {
                        Some(FnNode::Triple(
                            Box::new(FnNode::Number(a)),
                            Box::new(FnNode::Number(b)),
                            Box::new(FnNode::Number(c)),
                        ))
                    }
                    _ => None,
                }
            }
            FnNode::Random => {
                println!("Grammar not evaluated properly");
                None
            }
            FnNode::Rule(_) => {
                println!("Grammar not evaluated properly");
                None
            }
        }
    }

    pub fn expr_eval(&self, x: f32, y: f32) -> Option<Color> {
        // If Node is a triple call node_eval with that node or return None
        match self {
            FnNode::Triple(a, b, c) => {
                let a = match a.node_eval(x, y)? {
                    FnNode::Number(val) => val,
                    _ => return None,
                };
                let b = match b.node_eval(x, y)? {
                    FnNode::Number(val) => val,
                    _ => return None,
                };
                let c = match c.node_eval(x, y)? {
                    FnNode::Number(val) => val,
                    _ => return None,
                };
                Some(Color { r: a, g: b, b: c })
            }
            FnNode::If(cond, then, elze) => {
                let cond = match cond.node_eval(x, y)? {
                    FnNode::Number(val) => val,
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

    pub fn node_render(&self) {
        let h = FnNode::HEIGHT;
        let w = FnNode::WIDTH;
        let mut img = img::ImageBuffer::new(w, h);
        for y in 0..h {
            let ny = y as f32 / (h as f32) * 2.0 - 1.0;
            for x in 0..w {
                let nx = x as f32 / (w as f32) * 2.0 - 1.0;
                let color = self.expr_eval(nx, ny).unwrap();
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
