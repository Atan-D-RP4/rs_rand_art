use std::fmt::Display;

use image::{self as img};

const WIDTH: u32 = 1600;
const HEIGHT: u32 = 900;

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

// impl FnNode {
//     pub fn _is_terminal(&self) -> bool {
//         match self {
//             FnNode::Rule(_) => false,
//             FnNode::Add(a, b)
//             | FnNode::Sub(a, b)
//             | FnNode::Mul(a, b)
//             | FnNode::Div(a, b)
//             | FnNode::Mod(a, b)
//             | FnNode::Compare(a, _, b) => a._is_terminal() && b._is_terminal(),
//             FnNode::Triple(a, b, c) | FnNode::If(a, b, c) => {
//                 a._is_terminal() && b._is_terminal() && c._is_terminal()
//             }
//             _ => true,
//         }
//     }
// }

#[derive(Debug, Clone)]
pub struct Color {
    r: f64,
    g: f64,
    b: f64,
}

impl FnNode {
    const WIDTH: u32 = 400;
    const HEIGHT: u32 = 400;

    pub fn node_eval(&self, x: f64, y: f64 /*t: f64*/) -> Result<FnNode, String> {
        match self {
            FnNode::X => Ok(FnNode::Number(x)),
            FnNode::Y => Ok(FnNode::Number(y)),
            FnNode::T => Ok(FnNode::Number(x)),
            FnNode::Number(val) => Ok(FnNode::Number(*val)),
            FnNode::Boolean(val) => Ok(FnNode::Boolean(*val)),

            FnNode::Sqrt(expr) | FnNode::Abs(expr) | FnNode::Sin(expr) => {
                let expr = expr.node_eval(x, y)?;
                match expr {
                    FnNode::Number(val) => match self {
                        FnNode::Sqrt(_) => Ok(FnNode::Number(val.sqrt())),
                        FnNode::Abs(_) => Ok(FnNode::Number(val.abs())),
                        FnNode::Sin(_) => Ok(FnNode::Number(val.sin())),
                        _ => Err(format!("node_eval: Unary - {expr}")),
                    },
                    _ => Err(format!("node_eval: Unary - {expr}")),
                }
            }

            FnNode::Add(a, b) => {
                let a = a.node_eval(x, y)?;
                let b = b.node_eval(x, y)?;
                let err_str = format!("node_eval: Add - {a} {b}");
                match (a, b) {
                    (FnNode::Number(a), FnNode::Number(b)) => Ok(FnNode::Number(a + b)),
                    _ => Err(err_str),
                }
            }
            FnNode::Sub(a, b) => {
                let a = a.node_eval(x, y)?;
                let b = b.node_eval(x, y)?;
                let err_str = format!("node_eval: Sub - {a} {b}");
                match (a, b) {
                    (FnNode::Number(a), FnNode::Number(b)) => Ok(FnNode::Number(a - b)),
                    _ => Err(err_str),
                }
            }
            FnNode::Mul(a, b) => {
                let a = a.node_eval(x, y)?;
                let b = b.node_eval(x, y)?;
                let err_str = format!("node_eval: Mul - {a} {b}");
                match (a, b) {
                    (FnNode::Number(a), FnNode::Number(b)) => Ok(FnNode::Number(a * b)),
                    _ => Err(err_str),
                }
            }
            FnNode::Div(a, b) => {
                let a = a.node_eval(x, y)?;
                let b = b.node_eval(x, y)?;
                let err_str = format!("node_eval: Mul - {a} {b}");
                match (a, b) {
                    (FnNode::Number(a), FnNode::Number(b)) => Ok(FnNode::Number(a / b)),
                    _ => Err(err_str),
                }
            }
            FnNode::Mod(a, b) => {
                let a = a.node_eval(x, y)?;
                let b = b.node_eval(x, y)?;
                let err_str = format!("node_eval: Mul - {a} {b}");
                match (a, b) {
                    (FnNode::Number(a), FnNode::Number(b)) => Ok(FnNode::Number(a.rem_euclid(b))),
                    _ => Err(err_str),
                }
            }
            FnNode::Compare(a, ord, b) => {
                let a = a.node_eval(x, y)?;
                let b = b.node_eval(x, y)?;
                let err_str = format!("node_eval: Mul - {a} {b}");
                match ord {
                    CompareKind::GreaterThan => match (a, b) {
                        (FnNode::Number(a), FnNode::Number(b)) => Ok(FnNode::Boolean(a > b)),
                        _ => Err(err_str),
                    },
                    CompareKind::LessThan => match (a, b) {
                        (FnNode::Number(a), FnNode::Number(b)) => Ok(FnNode::Boolean(a < b)),
                        _ => Err(err_str),
                    },
                    CompareKind::GreaterThanEqual => match (a, b) {
                        (FnNode::Number(a), FnNode::Number(b)) => Ok(FnNode::Boolean(a >= b)),
                        _ => Err(err_str),
                    },
                    CompareKind::LessThanEqual => match (a, b) {
                        (FnNode::Number(a), FnNode::Number(b)) => Ok(FnNode::Boolean(a <= b)),
                        _ => Err(err_str),
                    },
                    CompareKind::Equal => match (a, b) {
                        (FnNode::Number(a), FnNode::Number(b)) => Ok(FnNode::Boolean(a == b)),
                        _ => Err(err_str),
                    },
                    CompareKind::NotEqual => match (a, b) {
                        (FnNode::Number(a), FnNode::Number(b)) => Ok(FnNode::Boolean(a != b)),
                        _ => Err(err_str),
                    },
                }
            }
            FnNode::If(cond, then, elze) => {
                let cond = cond.node_eval(x, y)?;
                match cond {
                    FnNode::Boolean(true) => then.node_eval(x, y),
                    FnNode::Boolean(false) => elze.node_eval(x, y),
                    _ => Err(format!("node_eval: If-elze - {cond} is not a boolean")),
                }
            }
            FnNode::Triple(a, b, c) => {
                let a = a.node_eval(x, y)?;
                let b = b.node_eval(x, y)?;
                let c = c.node_eval(x, y)?;
                let err_str = format!("node_eval: Triple - {a} {b} {c}");
                match (a, b, c) {
                    (FnNode::Number(a), FnNode::Number(b), FnNode::Number(c)) => {
                        Ok(FnNode::Triple(
                            Box::new(FnNode::Number(a)),
                            Box::new(FnNode::Number(b)),
                            Box::new(FnNode::Number(c)),
                        ))
                    }
                    _ => Err(err_str),
                }
            }
            FnNode::Random => Err(format!(
                "node_eval: Grammar not evaluated properly - Random"
            )),
            FnNode::Rule(_rule) => Err(format!(
                "node_eval: Grammar not evaluated properly - {_rule}"
            )),
        }
    }

    pub fn expr_eval(&self, x: f64, y: f64) -> Result<Color, String> {
        match self.node_eval(x, y)? {
            FnNode::Triple(a, b, c) => match (*a, *b, *c) {
                (FnNode::Number(r), FnNode::Number(g), FnNode::Number(b)) => Ok(Color { r, g, b }),
                _ => Err("expr_eval: Triple contains non-numbers".to_string()),
            },
            _ => Err("expr_eval: Not a triple node".to_string()),
        }
    }

    pub fn node_render(&self) {
        println!("Rendering node:\n{}", self);
        let h = FnNode::HEIGHT;
        let w = FnNode::WIDTH;
        let mut img = img::ImageBuffer::new(w, h);
        for y in 0..h {
            let ny = (y as f64 / (h as f64) * 2.0) - 1.0;
            for x in 0..w {
                let nx = (x as f64 / (w as f64) * 2.0) - 1.0;
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
            FnNode::If(cond, then, elze) => {
                write!(f, "if {} then\n\t{}\nelse\n\t{}\n", cond, then, elze)
            }
            // FnNode::Triple(a, b, c) => write!(f, "(\n\t{},\t\n\t{},\n\t{}\n)", a, b, c),
            FnNode::Triple(a, b, c) => write!(f, "({}, {}, {})", a, b, c),
            FnNode::Sqrt(expr) => write!(f, "sqrt({})", expr),
            FnNode::Abs(expr) => write!(f, "abs({})", expr),
            FnNode::Sin(expr) => write!(f, "sin({})", expr),
        }
    }
}
