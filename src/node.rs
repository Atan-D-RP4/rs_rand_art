use std::fmt::Display;

use image::{self as img};

const WIDTH: u32 = 400;
const HEIGHT: u32 = 400;

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
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
    T,
    Random,
    Boolean(bool),
    Number(f64),
    Rule(usize),

    Add(Box<FnNode>, Box<FnNode>),
    Sub(Box<FnNode>, Box<FnNode>),
    Mul(Box<FnNode>, Box<FnNode>),
    Div(Box<FnNode>, Box<FnNode>),
    Mod(Box<FnNode>, Box<FnNode>),
    Compare(Box<FnNode>, CompareKind, Box<FnNode>),

    If(Box<FnNode>, Box<FnNode>, Box<FnNode>),
    Triple(Box<FnNode>, Box<FnNode>, Box<FnNode>),
    Sqrt(Box<FnNode>),
    Abs(Box<FnNode>),
    Sin(Box<FnNode>),
    // Pow, Sin, Cos, Tan,
}

impl FnNode {
    pub fn _is_terminal(&self) -> bool {
        match self {
            FnNode::Rule(_) => false,
            FnNode::Add(a, b)
            | FnNode::Sub(a, b)
            | FnNode::Mul(a, b)
            | FnNode::Div(a, b)
            | FnNode::Mod(a, b)
            | FnNode::Compare(a, _, b) => a._is_terminal() && b._is_terminal(),
            FnNode::Triple(a, b, c) | FnNode::If(a, b, c) => {
                a._is_terminal() && b._is_terminal() && c._is_terminal()
            }
            _ => true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Color {
    r: f64,
    g: f64,
    b: f64,
}

fn split4(x: f64, y: f64) -> Color {
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

    pub fn node_eval(&self, x: f64, y: f64 /*t: f64*/) -> Option<FnNode> {
        match self {
            FnNode::X => Some(FnNode::Number(x)),
            FnNode::Y => Some(FnNode::Number(y)),
            FnNode::T => Some(FnNode::Number(x)),
            FnNode::Number(val) => Some(FnNode::Number(*val)),
            FnNode::Boolean(val) => Some(FnNode::Boolean(*val)),

            FnNode::Sqrt(expr) | FnNode::Abs(expr) | FnNode::Sin(expr) => {
                let expr = expr.node_eval(x, y)?;
                match expr {
                    FnNode::Number(val) => Some(FnNode::Number(val.sqrt())),
                    _ => None,
                }
            }

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
                        (FnNode::Number(a), FnNode::Number(b)) => Some(FnNode::Boolean(a > b)),
                        _ => None,
                    },
                    CompareKind::LessThan => match (a, b) {
                        (FnNode::Number(a), FnNode::Number(b)) => Some(FnNode::Boolean(a < b)),
                        _ => None,
                    },
                    CompareKind::GreaterThanEqual => match (a, b) {
                        (FnNode::Number(a), FnNode::Number(b)) => Some(FnNode::Boolean(a >= b)),
                        _ => None,
                    },
                    CompareKind::LessThanEqual => match (a, b) {
                        (FnNode::Number(a), FnNode::Number(b)) => Some(FnNode::Boolean(a <= b)),
                        _ => None,
                    },
                    CompareKind::Equal => match (a, b) {
                        (FnNode::Number(a), FnNode::Number(b)) => Some(FnNode::Boolean(a == b)),
                        _ => None,
                    },
                    CompareKind::NotEqual => match (a, b) {
                        (FnNode::Number(a), FnNode::Number(b)) => Some(FnNode::Boolean(a != b)),
                        _ => None,
                    },
                }
            }
            FnNode::If(cond, then, elze) => {
                let cond = cond.node_eval(x, y)?;
                match cond {
                    FnNode::Boolean(true) => then.node_eval(x, y),
                    FnNode::Boolean(false) => elze.node_eval(x, y),
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

    pub fn expr_eval(&self, x: f64, y: f64) -> Option<Color> {
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
        println!("Rendering node: {}", self);
        let h = FnNode::HEIGHT;
        let w = FnNode::WIDTH;
        let mut img = img::ImageBuffer::new(w, h);
        for y in 0..h {
            let ny = y as f64 / (h as f64) * 2.0 - 1.0;
            for x in 0..w {
                let nx = x as f64 / (w as f64) * 2.0 - 1.0;
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

impl Display for FnNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FnNode::X => write!(f, "X"),
            FnNode::Y => write!(f, "Y"),
            FnNode::T => write!(f, "T"),
            FnNode::Random => write!(f, "rand"),
            FnNode::Boolean(val) => write!(f, "{}", val),
            FnNode::Number(val) => write!(f, "{}", val),
            FnNode::Rule(val) => write!(f, "{}", val),
            FnNode::Add(a, b) => write!(f, "Add({}, {})", a, b),
            FnNode::Sub(a, b) => write!(f, "Sub({}, {})", a, b),
            FnNode::Mul(a, b) => write!(f, "Mul({}, {})", a, b),
            FnNode::Div(a, b) => write!(f, "Div({}, {})", a, b),
            FnNode::Mod(a, b) => write!(f, "Mod({}, {})", a, b),
            FnNode::Compare(a, ord, b) => write!(f, "({} {:?} {})", a, ord, b),
            FnNode::If(cond, then, elze) => write!(f, "if {} then{\n\t} else {}\n", cond, then, elze),
            FnNode::Triple(a, b, c) => write!(f, "(\n\t{},\t\n\t{},\n\t{}\n)", a, b, c),
            FnNode::Sqrt(expr) => write!(f, "sqrt({})", expr),
            FnNode::Abs(expr) => write!(f, "abs({})", expr),
            FnNode::Sin(expr) => write!(f, "sin({})", expr),
        }
    }
}
