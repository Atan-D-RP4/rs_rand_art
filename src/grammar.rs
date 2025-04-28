use crate::node::FnNode;
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Branch {
    pub node: FnNode,
    pub weight: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Rule {
    pub branches: Vec<Branch>,
    pub weight_sum: usize,
}

#[derive(Debug, Clone)]
pub struct Grammar {
    pub symbols: Vec<String>,
    pub rules: Vec<Rule>,
    pub map: HashMap<String, Rule>,
}

pub struct GrammarError {
    pub message: String,
    pub range: std::ops::Range<usize>,
}

pub struct RuleError {
    pub message: String,
    pub range: std::ops::Range<usize>,
}

// =============================================================================
impl Branch {
    pub fn new(node: FnNode, weight: usize) -> Self {
        Branch { node, weight }
    }
}

impl Rule {
    pub fn new(branches: Vec<Branch>) -> Self {
        let weight_sum = branches.iter().map(|b| b.weight).sum();
        Rule { branches, weight_sum }
    }
}

#[allow(dead_code)]
impl Grammar {
    pub fn new() -> Self {
        Grammar {
            symbols: vec![],
            rules: vec![],
            map: HashMap::new(),
        }
    }

    pub fn add_rule(&mut self, branches: Vec<Branch>, symbol: String) -> Result<(), &'static str> {
        if branches.is_empty() {
            return Err("Empty rule branches");
        }

        let weight_sum = branches.iter().map(|b| b.weight).sum();
        let rule = Rule {
            branches,
            weight_sum,
        };
        self.rules.push(rule.clone());
        self.symbols.push(symbol.clone());
        self.map.insert(symbol, rule);
        Ok(())
    }

    pub fn gen_from_rule(&self, rule_idx: usize, depth: usize) -> Option<FnNode> {
        if depth == 0 || rule_idx >= self.rules.len() {
            return None;
        }

        let rule = &self.rules.get(rule_idx).or_else(|| {
            eprintln!("Rule index out of bounds: {rule_idx}");
            None
        })?;
        let mut attempts: i32 = 100; // GEN_RULE_MAX_ATTEMPTS

        while attempts > 0 {
            use rand::Rng;
            let p = rand::thread_rng().gen_range(0.0..1.0);
            let mut t = 0.0;

            for branch in &rule.branches {
                t += branch.weight as f64 / rule.weight_sum as f64;

                if t >= p {
                    let node = self.gen_node(&branch.node, depth);
                    match node {
                        Some(node) => return Some(node),
                        None => break,
                    }
                }
            }
            attempts = attempts.checked_sub(1).unwrap_or(0);
        }
        None
    }

    pub fn gen_node(&self, node: &FnNode, depth: usize) -> Option<FnNode> {
        match node {
            // Terminal nodes
            FnNode::X | FnNode::Y | FnNode::T | FnNode::Number(_) | FnNode::Boolean(_) => {
                Some(node.clone())
            }

            // Random number generation
            FnNode::Random => {
                use rand::Rng;
                Some(FnNode::Number(
                    rand::thread_rng().gen_range(0.0..1.0) * 2.0 - 1.0,
                ))
            }

            // Unary operations
            FnNode::Unary(op, expr) => {
                let e = self.gen_node(expr, depth)?;
                Some(FnNode::Unary(op.clone(), Box::new(e)))
            }

            // Binary operations
            FnNode::Arithmetic(lhs, _, rhs) | FnNode::Compare(lhs, _, rhs) => {
                let l = self.gen_node(lhs, depth)?;
                let r = self.gen_node(rhs, depth)?;
                Some(match node {
                    FnNode::Arithmetic(_, kind, _) => {
                        FnNode::Arithmetic(Box::new(l), kind.clone(), Box::new(r))
                    }
                    FnNode::Compare(_, kind, _) => {
                        FnNode::Compare(Box::new(l), kind.clone(), Box::new(r))
                    }
                    _ => unreachable!(),
                })
            }

            // Triple operation
            FnNode::Triple(first, second, third) | FnNode::If(first, second, third) => {
                let f = self.gen_node(first, depth)?;
                let s = self.gen_node(second, depth)?;
                let t = self.gen_node(third, depth)?;
                match node {
                    FnNode::Triple(_, _, _) => {
                        Some(FnNode::Triple(Box::new(f), Box::new(s), Box::new(t)))
                    }
                    FnNode::If(_, _, _) => Some(FnNode::If(Box::new(f), Box::new(s), Box::new(t))),
                    _ => unreachable!(),
                }
            }

            // Rule reference
            FnNode::Rule(rule_idx) => self.gen_from_rule(*rule_idx, depth - 1),
        }
    }
}

impl Default for Grammar {
    fn default() -> Self {
        use crate::node::{ArithmeticOp, FnNode /*CompareOp*/, UnaryOp};
        let e = 0;
        let a = 1;
        let c = 2;
        /*
        * # Entry
        * E | vec3(C, C, C)
        *   | vec3(A, C, A)
        *   ;

        * # Terminal
        * A | random
        *   | x
        *   | y
        *   | t
        *   | abs(x)
        *   | abs(y)
        *   | sqrt(add(mult(x, x), mult(y, y))) # Distance from (0, 0) to (x, y)
        *   ;

        * # Expressions
        * C ||  A
        *   ||| add(C, C)
        *   ||| mult(C, C)
        *   ||| abs(C)
        *   #| sqrt(abs(C))
        *   #||| sin(C)
        *   ;

        **/

        let mut grammar = Grammar::new();
        let _ = grammar.add_rule(
            vec![
                Branch::new(
                    FnNode::triple(FnNode::Rule(c), FnNode::Rule(c), FnNode::Rule(c)),
                    1,
                ),
                Branch::new(
                    FnNode::triple(FnNode::Rule(a), FnNode::Rule(c), FnNode::Rule(a)),
                    1,
                ),
            ],
            "E".to_string(),
        );
        let _ = grammar.add_rule(
            vec![
                Branch::new(FnNode::Random, 1),
                Branch::new(FnNode::X, 1),
                Branch::new(FnNode::Y, 1),
                Branch::new(FnNode::T, 1),
                Branch::new(
                    FnNode::unary(
                        UnaryOp::Sqrt,
                        FnNode::arithmetic(
                            FnNode::arithmetic(FnNode::X, ArithmeticOp::Mul, FnNode::X),
                            ArithmeticOp::Add,
                            FnNode::arithmetic(FnNode::Y, ArithmeticOp::Mul, FnNode::Y),
                        ),
                    ),
                    2,
                ),
            ],
            "A".to_string(),
        );
        let _ = grammar.add_rule(
            vec![
                Branch::new(FnNode::Rule(a), 2),
                Branch::new(
                    FnNode::arithmetic(FnNode::Rule(c), ArithmeticOp::Add, FnNode::Rule(c)),
                    3,
                ),
                Branch::new(
                    FnNode::arithmetic(FnNode::Rule(c), ArithmeticOp::Mul, FnNode::Rule(c)),
                    3,
                ),
                Branch::new(
                    FnNode::unary(UnaryOp::Sqrt, FnNode::unary(UnaryOp::Abs, FnNode::Rule(c))),
                    3,
                ),
            ],
            "C".to_string(),
        );
        grammar
    }
}

impl Display for Grammar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (symbol, rule) in &self.map {
            write!(f, "\n{symbol}: ")?;
            for (jdx, branch) in rule.branches.iter().enumerate() {
                if jdx != 0 {
                    write!(f, " | ")?;
                }
                write!(f, "{} [{}]", branch.node, branch.weight)?;
            }
        }
        Ok(())
    }
}
