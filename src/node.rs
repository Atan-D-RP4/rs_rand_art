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

use std::fmt::Display;

use image::{self as img, buffer};

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 944;

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Hash)]
pub enum CompareOp {
    GreaterThan,
    LessThan,
    GreaterThanEqual,
    LessThanEqual,
    Equal,
    NotEqual,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ArithmeticOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum UnaryOp {
    Sqrt,
    Abs,
    Sin,
    Cos,
    Tan,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum FnNode {
    X,
    Y,
    T,
    Random,
    Boolean(bool),
    Number(f32),
    Rule(usize),

    Arithmetic(Box<FnNode>, ArithmeticOp, Box<FnNode>),
    Compare(Box<FnNode>, CompareOp, Box<FnNode>),
    Unary(UnaryOp, Box<FnNode>),

    If(Box<FnNode>, Box<FnNode>, Box<FnNode>),
    Triple(Box<FnNode>, Box<FnNode>, Box<FnNode>),
}

#[allow(dead_code)]
impl FnNode {
    pub fn number(n: f32) -> FnNode {
        FnNode::Number(n)
    }

    pub fn arithmetic(a: FnNode, kind: ArithmeticOp, b: FnNode) -> FnNode {
        FnNode::Arithmetic(Box::new(a), kind, Box::new(b))
    }

    pub fn compare(a: FnNode, kind: CompareOp, b: FnNode) -> FnNode {
        FnNode::Compare(Box::new(a), kind, Box::new(b))
    }

    pub fn unary(op: UnaryOp, expr: FnNode) -> FnNode {
        FnNode::Unary(op, Box::new(expr))
    }

    pub fn if_(cond: FnNode, then_branch: FnNode, else_branch: FnNode) -> FnNode {
        FnNode::If(Box::new(cond), Box::new(then_branch), Box::new(else_branch))
    }

    pub fn triple(r: FnNode, g: FnNode, b: FnNode) -> FnNode {
        FnNode::Triple(Box::new(r), Box::new(g), Box::new(b))
    }
}

#[derive(Debug, Clone)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
}

impl FnNode {
    const OPTIMIZED: bool = false;
    pub fn optimize(&mut self) -> Result<(), String> {
        if Self::OPTIMIZED {
            return Ok(());
        }
        match self {
            FnNode::Number(val) if val.is_nan() => {
                eprintln!("NaN encountered during optimization");
                *self = FnNode::Number(0.0);
                Ok(())
            }
            FnNode::X | FnNode::Y | FnNode::T | FnNode::Boolean(_) | FnNode::Number(_) => Ok(()),

            FnNode::Random | FnNode::Rule(_) => {
                Err("Rule node encountered during optimization".to_string())
            }

            FnNode::Triple(first, second, third) => {
                first.optimize()?;
                second.optimize()?;
                third.optimize()?;
                Ok(())
            }

            FnNode::Arithmetic(a, _, b) => {
                a.optimize()?;
                b.optimize()?;
                match ((**a).clone(), (**b).clone()) {
                    (FnNode::Number(a), FnNode::Number(b)) => {
                        *self = FnNode::Number(match self {
                            FnNode::Arithmetic(_, ArithmeticOp::Add, _) => a + b,
                            FnNode::Arithmetic(_, ArithmeticOp::Sub, _) => a - b,
                            FnNode::Arithmetic(_, ArithmeticOp::Mul, _) => a * b,
                            FnNode::Arithmetic(_, ArithmeticOp::Div, _) => a / b,
                            FnNode::Arithmetic(_, ArithmeticOp::Mod, _) => a % b,
                            _ => {
                                return Err("Invalid operands for arithmetic operation".to_string())
                            }
                        });
                        Ok(())
                    }
                    _ => Ok(()),
                }
            }

            FnNode::Compare(a, _, b) => {
                a.optimize()?;
                b.optimize()?;
                match ((**a).clone(), (**b).clone()) {
                    (FnNode::Number(a), FnNode::Number(b)) => {
                        *self = FnNode::Boolean(match self {
                            FnNode::Compare(_, CompareOp::GreaterThan, _) => a > b,
                            FnNode::Compare(_, CompareOp::LessThan, _) => a < b,
                            FnNode::Compare(_, CompareOp::GreaterThanEqual, _) => a >= b,
                            FnNode::Compare(_, CompareOp::LessThanEqual, _) => a <= b,
                            FnNode::Compare(_, CompareOp::Equal, _) => a == b,
                            FnNode::Compare(_, CompareOp::NotEqual, _) => a != b,
                            _ => {
                                return Err("Invalid operands for comparison operation".to_string())
                            }
                        });
                        Ok(())
                    }
                    _ => Ok(()),
                }
            }

            FnNode::Unary(_, expr) => {
                expr.optimize()?;
                match **expr {
                    FnNode::Number(val) => {
                        *self = FnNode::Number(match self {
                            FnNode::Unary(UnaryOp::Sqrt, _) => val.sqrt(),
                            FnNode::Unary(UnaryOp::Abs, _) => val.abs(),
                            FnNode::Unary(UnaryOp::Sin, _) => val.sin(),
                            FnNode::Unary(UnaryOp::Cos, _) => val.cos(),
                            FnNode::Unary(UnaryOp::Tan, _) => val.tan(),
                            _ => return Err("Invalid operand for unary operation".to_string()),
                        });
                        Ok(())
                    }
                    _ => Ok(()),
                }
            }

            FnNode::If(cond, then_branch, else_branch) => {
                cond.optimize()?;
                then_branch.optimize()?;
                else_branch.optimize()?;
                match **cond {
                    FnNode::Boolean(true) => {
                        *self = *then_branch.clone();
                        Ok(())
                    }
                    FnNode::Boolean(false) => {
                        *self = *else_branch.clone();
                        Ok(())
                    }
                    _ => Ok(()),
                }
            }
        }
    }

    fn eval(&self, x: f32, y: f32, t: f32) -> Result<FnNode, String> {
        match self {
            FnNode::X => Ok(FnNode::Number(x)),
            FnNode::Y => Ok(FnNode::Number(y)),
            FnNode::T => Ok(FnNode::Number(t)),
            FnNode::Boolean(val) => Ok(FnNode::Boolean(*val)),
            FnNode::Number(val) => Ok(FnNode::Number(*val)),
            FnNode::Random | FnNode::Rule(_) => {
                Err("Rule node encountered during evaluation".to_string())
            }

            FnNode::Arithmetic(a, op, b) => {
                let a = a.eval(x, y, t)?;
                let b = b.eval(x, y, t)?;

                match (a, b) {
                    (FnNode::Number(a), FnNode::Number(b)) => match op {
                        ArithmeticOp::Add => Ok(FnNode::Number(a + b)),
                        ArithmeticOp::Sub => Ok(FnNode::Number(a - b)),
                        ArithmeticOp::Mul => Ok(FnNode::Number(a * b)),
                        ArithmeticOp::Div => Ok(FnNode::Number(a / b)),
                        ArithmeticOp::Mod => Ok(FnNode::Number(a % b)),
                    },
                    _ => Err("Invalid operands for arithmetic operation".to_string()),
                }
            }

            FnNode::Compare(a, ord, b) => {
                let a = a.eval(x, y, t)?;
                let b = b.eval(x, y, t)?;

                match (a, b) {
                    (FnNode::Number(a), FnNode::Number(b)) => match ord {
                        CompareOp::GreaterThan => Ok(FnNode::Boolean(a > b)),
                        CompareOp::LessThan => Ok(FnNode::Boolean(a < b)),
                        CompareOp::GreaterThanEqual => Ok(FnNode::Boolean(a >= b)),
                        CompareOp::LessThanEqual => Ok(FnNode::Boolean(a <= b)),
                        CompareOp::Equal => Ok(FnNode::Boolean(a == b)),
                        CompareOp::NotEqual => Ok(FnNode::Boolean(a != b)),
                    },
                    _ => Err("Invalid operands for comparison operation".to_string()),
                }
            }

            FnNode::Unary(op, expr) => {
                let expr = expr.eval(x, y, t)?;
                match op {
                    UnaryOp::Sqrt => match expr {
                        FnNode::Number(val) => Ok(FnNode::Number(val.sqrt())),
                        _ => Err("Invalid operand for sqrt operation".to_string()),
                    },
                    UnaryOp::Abs => match expr {
                        FnNode::Number(val) => Ok(FnNode::Number(val.abs())),
                        _ => Err("Invalid operand for abs operation".to_string()),
                    },
                    UnaryOp::Sin => match expr {
                        FnNode::Number(val) => Ok(FnNode::Number(val.sin())),
                        _ => Err("Invalid operand for sin operation".to_string()),
                    },
                    UnaryOp::Cos => match expr {
                        FnNode::Number(val) => Ok(FnNode::Number(val.cos())),
                        _ => Err("Invalid operand for cos operation".to_string()),
                    },
                    UnaryOp::Tan => match expr {
                        FnNode::Number(val) => Ok(FnNode::Number(val.tan())),
                        _ => Err("Invalid operand for tan operation".to_string()),
                    },
                }
            }

            FnNode::If(cond, then_branch, else_branch) => match cond.eval(x, y, t)? {
                FnNode::Boolean(true) => then_branch.eval(x, y, t),
                FnNode::Boolean(false) => else_branch.eval(x, y, t),
                _ => Err("Invalid condition for if statement".to_string()),
            },

            FnNode::Triple(first, second, third) => {
                let first = first.eval(x, y, t)?;
                let second = second.eval(x, y, t)?;
                let third = third.eval(x, y, t)?;
                match (first, second, third) {
                    (FnNode::Number(r), FnNode::Number(g), FnNode::Number(b)) => {
                        Ok(FnNode::Triple(
                            Box::new(FnNode::Number(r)),
                            Box::new(FnNode::Number(g)),
                            Box::new(FnNode::Number(b)),
                        ))
                    }
                    _ => Err("Invalid operands for triple operation".to_string()),
                }
            }
        }
    }

    fn eval_fn(&self, x: f32, y: f32, t: f32) -> Result<Color, String> {
        match self.eval(x, y, t) {
            Ok(FnNode::Triple(r, g, b)) => match (*r, *g, *b) {
                (FnNode::Number(r), FnNode::Number(g), FnNode::Number(b)) => Ok(Color { r, g, b }),
                _ => Err("Invalid operands for triple operation".to_string()),
            },
            _ => Err("Invalid result for function".to_string()),
        }
    }

    pub fn render(&self) -> Result<(), String> {
        let mut img = img::ImageBuffer::new(WIDTH, HEIGHT);

        for y in 0..HEIGHT {
            let ny = (y as f32 / HEIGHT as f32) * 2.0 - 1.0;
            for x in 0..WIDTH {
                let nx = (x as f32 / WIDTH as f32) * 2.0 - 1.0;

                let color = self.eval_fn(nx, ny, 0.0)?;
                let pixel = img::Rgb([
                    ((color.r + 1.0) / 2.0 * 255.0) as u8,
                    ((color.g + 1.0) / 2.0 * 255.0) as u8,
                    ((color.b + 1.0) / 2.0 * 255.0) as u8,
                ]);

                img.put_pixel(x, y, pixel);
            }
        }

        img.save("output.png").map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn compile_to_glsl_fs(&mut self) -> Result<String, String> {
        self.optimize()?;

        let mut default_fs = String::from(
            r#"
#version 330

in vec2 fragTexCoord;
out vec4 finalColor;
uniform float time;

vec4 map_rgb(vec3 rgb) {
    return vec4(rgb + 1/2, 1);
}

void main() {
    float x = fragTexCoord.x;
    float y = fragTexCoord.y;
    float t = tan(time);
    finalColor = map_rgb(%s);
}
        "#,
        );

        let mut compiled_node = String::new();
        match self.compile_to_glsl_fs_expr(&mut compiled_node) {
            Ok(_) => {
                default_fs = default_fs.replace("%s", &compiled_node.as_str());
                Ok(default_fs.to_string())
            }
            Err(e) => Err(e),
        }
    }

    fn compile_to_glsl_fs_expr(&self, buffer: &mut String) -> Result<(), String> {
        match self {
            FnNode::X => buffer.push_str("x"),
            FnNode::Y => buffer.push_str("y"),
            FnNode::T => buffer.push_str("t"),
            FnNode::Number(val) => buffer.push_str(&format!("({})", val.to_string())),
            FnNode::Boolean(val) => match val {
                true => buffer.push_str("true"),
                false => buffer.push_str("false"),
            },

            FnNode::Random | FnNode::Rule(_) => {
                return Err("Rule node encountered during GLSL compilation".to_string())
            }

            FnNode::Unary(kind, expr) => {
                buffer.push_str(match kind {
                    UnaryOp::Sqrt => "sqrt(",
                    UnaryOp::Abs => "abs(",
                    UnaryOp::Sin => "sin(",
                    UnaryOp::Cos => "cos(",
                    UnaryOp::Tan => "tan(",
                });
                expr.compile_to_glsl_fs_expr(buffer)?;
                buffer.push_str(")");
            }

            FnNode::Arithmetic(a, kind, b) => {
                buffer.push_str("(");
                if let ArithmeticOp::Mod = kind {
                    buffer.push_str("mod(");
                }
                a.compile_to_glsl_fs_expr(buffer)?;
                buffer.push_str(match kind {
                    ArithmeticOp::Add => " + ",
                    ArithmeticOp::Sub => " - ",
                    ArithmeticOp::Mul => " * ",
                    ArithmeticOp::Div => " / ",
                    ArithmeticOp::Mod => ", ",
                });
                b.compile_to_glsl_fs_expr(buffer)?;
                if let ArithmeticOp::Mod = kind {
                    buffer.push_str(")");
                }
                buffer.push_str(")");
            }

            FnNode::Compare(a, kind, b) => {
                buffer.push_str("(");
                a.compile_to_glsl_fs_expr(buffer)?;
                buffer.push_str(match kind {
                    CompareOp::GreaterThanEqual => " >= ",
                    CompareOp::GreaterThan => " > ",
                    CompareOp::LessThanEqual => " <= ",
                    CompareOp::LessThan => " < ",
                    CompareOp::Equal => " == ",
                    CompareOp::NotEqual => " != ",
                });
                b.compile_to_glsl_fs_expr(buffer)?;
                buffer.push_str(")");
            }

            FnNode::If(cond, then, elze) => {
                buffer.push_str("((");
                cond.compile_to_glsl_fs_expr(buffer)?;
                buffer.push_str(") ? (");
                then.compile_to_glsl_fs_expr(buffer)?;
                buffer.push_str(") : (");
                elze.compile_to_glsl_fs_expr(buffer)?;
                buffer.push_str("))");
            }

            FnNode::Triple(r, g, b) => {
                buffer.push_str("vec3(");
                r.compile_to_glsl_fs_expr(buffer)?;
                buffer.push_str(", ");
                g.compile_to_glsl_fs_expr(buffer)?;
                buffer.push_str(", ");
                b.compile_to_glsl_fs_expr(buffer)?;
                buffer.push_str(")");
            }
        }
        Ok(())
    }
}

impl FnNode {
    #[allow(dead_code)]
    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: usize) -> std::fmt::Result {
        let indent_str = "  ".repeat(indent);

        match self {
            // Leaf nodes
            FnNode::X => writeln!(f, "{}X", indent_str),
            FnNode::Y => writeln!(f, "{}Y", indent_str),
            FnNode::T => writeln!(f, "{}T", indent_str),
            FnNode::Random => writeln!(f, "{}Random", indent_str),
            FnNode::Boolean(val) => writeln!(f, "{}Boolean({})", indent_str, val),
            FnNode::Number(val) => writeln!(f, "{}Number({})", indent_str, val),
            FnNode::Rule(val) => writeln!(f, "{}Rule({})", indent_str, val),

            // Binary operations
            FnNode::Arithmetic(a, op, b) => {
                writeln!(f, "{}Arithmetic({:?})", indent_str, op)?;
                a.fmt_with_indent(f, indent + 1)?;
                b.fmt_with_indent(f, indent + 1)
            }

            // Comparison
            FnNode::Compare(a, ord, b) => {
                writeln!(f, "{}Compare({:?})", indent_str, ord)?;
                a.fmt_with_indent(f, indent + 1)?;
                b.fmt_with_indent(f, indent + 1)
            }

            FnNode::Unary(op, expr) => {
                writeln!(f, "{}Unary({:?})", indent_str, op)?;
                expr.fmt_with_indent(f, indent + 1)
            }

            // Control flow
            FnNode::If(cond, then_branch, else_branch) => {
                writeln!(f, "{}If", indent_str)?;
                writeln!(f, "{}Condition:", indent_str)?;
                cond.fmt_with_indent(f, indent + 1)?;
                writeln!(f, "{}Then:", indent_str)?;
                then_branch.fmt_with_indent(f, indent + 1)?;
                writeln!(f, "{}Else:", indent_str)?;
                else_branch.fmt_with_indent(f, indent + 1)
            }

            // Triple for colors
            FnNode::Triple(r, g, b) => {
                writeln!(f, "{}Triple", indent_str)?;
                r.fmt_with_indent(f, indent + 1)?;
                g.fmt_with_indent(f, indent + 1)?;
                b.fmt_with_indent(f, indent + 1)
            }
        }
    }
}

impl Display for FnNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_with_indent(f, 0)
    }
    // fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    //     match self {
    //         FnNode::X => write!(f, "x"),
    //         FnNode::Y => write!(f, "y"),
    //         FnNode::T => write!(f, "t"),
    //         FnNode::Random => write!(f, "random"),
    //         FnNode::Boolean(val) => write!(f, "{}", val),
    //         FnNode::Number(val) => write!(f, "{}", val),
    //         FnNode::Rule(val) => write!(f, "rule({})", val),
    //         FnNode::Arithmetic(a, op, b) => match op {
    //             ArithmeticOp::Add => write!(f, "add({}, {})", a, b),
    //             ArithmeticOp::Sub => write!(f, "sub({}, {})", a, b),
    //             ArithmeticOp::Mul => write!(f, "mul({}, {})", a, b),
    //             ArithmeticOp::Div => write!(f, "div({}, {})", a, b),
    //             ArithmeticOp::Mod => write!(f, "mod({}, {})", a, b),
    //         },
    //         FnNode::Compare(a, ord, b) => match ord {
    //             CompareOp::GreaterThan => write!(f, "({} gt {})", a, b),
    //             CompareOp::LessThan => write!(f, "({} lt {})", a, b),
    //             CompareOp::GreaterThanEqual => write!(f, "({} gte {})", a, b),
    //             CompareOp::LessThanEqual => write!(f, "({} lte {})", a, b),
    //             CompareOp::Equal => write!(f, "({} eq {})", a, b),
    //             CompareOp::NotEqual => write!(f, "({} neq {})", a, b),
    //         },
    //         FnNode::Unary(op, expr) => match op {
    //             UnaryOp::Sqrt => write!(f, "sqrt({})", expr),
    //             UnaryOp::Abs => write!(f, "abs({})", expr),
    //             UnaryOp::Sin => write!(f, "sin({})", expr),
    //             UnaryOp::Cos => write!(f, "cos({})", expr),
    //             UnaryOp::Tan => write!(f, "tan({})", expr),
    //         },
    //         FnNode::If(cond, then_branch, else_branch) => {
    //             write!(f, "if({}, {}, {})", cond, then_branch, else_branch)
    //         }
    //         FnNode::Triple(r, g, b) => write!(f, "({}, {}, {})", r, g, b),
    //     }
    // }
}
