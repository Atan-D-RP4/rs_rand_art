use crate::node::FnNode;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct GrammarBranch {
    pub node: FnNode,
    pub weight: usize,
}

#[derive(Debug, Clone)]
pub struct Rule {
    symbol: &'static str,
    branches: Vec<GrammarBranch>,
    weight_sum: usize,
}

#[derive(Debug, Clone)]
pub struct Grammar {
    pub rules: Vec<Rule>,
}

impl Grammar {
    pub fn new() -> Self {
        Grammar { rules: vec![] }
    }

    pub fn add_rule(
        &mut self,
        branches: Vec<GrammarBranch>,
        symbol: &'static str,
    ) -> Result<(), &'static str> {
        if branches.is_empty() {
            return Err("Empty rule branches");
        }

        let weight_sum = branches.iter().map(|b| b.weight).sum();
        self.rules.push(Rule {
            symbol,
            branches,
            weight_sum,
        });
        Ok(())
    }

    pub fn gen_rule(&self, rule_idx: usize, depth: usize) -> Option<FnNode> {
        if depth == 0 || rule_idx >= self.rules.len() {
            return None;
        }

        let rule = &self.rules[rule_idx];
        let mut attempts = 100; // GEN_RULE_MAX_ATTEMPTS

        while attempts > 0 {
            let p = rand::random::<f32>();
            let mut t = 0.0;

            for branch in &rule.branches {
                t += (branch.weight as f32 / rule.weight_sum as f32) as f32;

                if t >= p {
                    let node = self.gen_node(&branch.node, depth);
                    match node {
                        Some(node) => return Some(node),
                        None => break,
                    }
                }
            }
            attempts -= 1;
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
            FnNode::Random => Some(FnNode::Number(rand::random::<f64>() * 2.0 - 1.0)),

            // Unary operations
            FnNode::Sqrt(expr) | FnNode::Abs(expr) | FnNode::Sin(expr) => {
                self.gen_node(expr, depth - 1).map(|n| match node {
                    FnNode::Sqrt(_) => FnNode::Sqrt(Box::new(n)),
                    FnNode::Abs(_) => FnNode::Abs(Box::new(n)),
                    FnNode::Sin(_) => FnNode::Sin(Box::new(n)),
                    _ => unreachable!(),
                })
            }

            // Binary operations
            FnNode::Add(lhs, rhs)
            | FnNode::Sub(lhs, rhs)
            | FnNode::Mul(lhs, rhs)
            | FnNode::Div(lhs, rhs)
            | FnNode::Mod(lhs, rhs)
            | FnNode::Compare(lhs, _, rhs) => {
                let l = self.gen_node(lhs, depth)?;
                let r = self.gen_node(rhs, depth)?;
                Some(match node {
                    FnNode::Add(_, _) => FnNode::Add(Box::new(l), Box::new(r)),
                    FnNode::Sub(_, _) => FnNode::Sub(Box::new(l), Box::new(r)),
                    FnNode::Mul(_, _) => FnNode::Mul(Box::new(l), Box::new(r)),
                    FnNode::Div(_, _) => FnNode::Div(Box::new(l), Box::new(r)),
                    FnNode::Mod(_, _) => FnNode::Mod(Box::new(l), Box::new(r)),
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
            FnNode::Rule(rule_idx) => self.gen_rule(*rule_idx, depth - 1),
        }
    }
}

impl Display for Grammar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (idx, rule) in self.rules.iter().enumerate() {
            if idx != 0 {
                write!(f, "\n")?;
            }
            write!(f, "{} {}: ", idx,  rule.symbol)?;
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
