use crate::node::FnNode;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Grammar {
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub branches: Vec<(FnNode, f32)>,
}

impl Grammar {
    pub fn new() -> Self {
        Grammar { rules: vec![] }
    }

    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    pub fn gen_rule(&self, rule_idx: usize, depth: usize) -> Result<FnNode, &str> {
        println!("Grammar:\n{}", self);
        println!("Rule: {}", rule_idx);
        if !(rule_idx < self.rules.len()) {
            return Err("Invalid Rule")
        }

        use rand::Rng;
        let rand = rand::thread_rng().gen_range(0..10);
        let branches = &self.rules[rule_idx].branches;
        if branches.len() == 0 {
            println!("Invalid Grammar");
            return Err("Invalid Grammar");
        }

        let idx = rand % branches.len();
        self.gen_node(branches[idx].0.clone(), depth)
    }

    pub fn gen_node(&self, node: FnNode, depth: usize) -> Result<FnNode, &str> {
        match node {
            FnNode::Rule(entry) => self.gen_rule(entry, depth),
            FnNode::Random => {
                use rand::Rng;
                let rand = rand::thread_rng().gen_range(0.0..1.0);
                let rand = (rand * 2.0) - 1.0;
                Ok(FnNode::Number(rand))
            }

            FnNode::Add(a, b) => {
                let a = self.gen_node(*a, depth)?;
                let b = self.gen_node(*b, depth)?;
                Ok(FnNode::Add(Box::new(a), Box::new(b)))
            }
            FnNode::Sub(a, b) => {
                let a = self.gen_node(*a, depth)?;
                let b = self.gen_node(*b, depth)?;
                Ok(FnNode::Sub(Box::new(a), Box::new(b)))
            }
            FnNode::Mul(a, b) => {
                let a = self.gen_node(*a, depth)?;
                let b = self.gen_node(*b, depth)?;
                Ok(FnNode::Mul(Box::new(a), Box::new(b)))
            }
            FnNode::Div(a, b) => {
                let a = self.gen_node(*a, depth)?;
                let b = self.gen_node(*b, depth)?;
                Ok(FnNode::Div(Box::new(a), Box::new(b)))
            }
            FnNode::Mod(a, b) => {
                let a = self.gen_node(*a, depth)?;
                let b = self.gen_node(*b, depth)?;
                Ok(FnNode::Mod(Box::new(a), Box::new(b)))
            }
            FnNode::Compare(a, ord, b) => {
                let a = self.gen_node(*a, depth)?;
                let b = self.gen_node(*b, depth)?;
                Ok(FnNode::Compare(Box::new(a), ord, Box::new(b)))
            }

            FnNode::Triple(a, b, c) => {
                let a = self.gen_node(*a, depth)?;
                let b = self.gen_node(*b, depth)?;
                let c = self.gen_node(*c, depth)?;
                Ok(FnNode::Triple(Box::new(a), Box::new(b), Box::new(c)))
            }
            FnNode::If(cond, then, elze) => {
                let a = self.gen_node(*cond, depth)?;
                let b = self.gen_node(*then, depth)?;
                let c = self.gen_node(*elze, depth)?;
                Ok(FnNode::If(Box::new(a), Box::new(b), Box::new(c)))
            }

            _ => Ok(node),
        }
    }
}

impl Display for Grammar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (idx, rule) in self.rules.iter().enumerate() {
            if idx != 0 {
                write!(f, "\n")?;
            }
            write!(f, "Rule {}: ", idx)?;
            for (idx, (node, prob)) in rule.branches.iter().enumerate() {
                if idx != 0 {
                    write!(f, " | ")?;
                }
                write!(f, "({:?}, [{:.2}])", node, prob)?;
            }
        }
        Ok(())
    }
}
