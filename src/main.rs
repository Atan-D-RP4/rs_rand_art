mod grammar;
mod node;
mod simple;

fn test1() {
    use crate::node::{CompareKind, FnNode};
    // Grayscale node
    let grayscale_node = FnNode::Triple(
        Box::new(FnNode::X),
        Box::new(FnNode::X),
        Box::new(FnNode::X),
    );

    let mistake = FnNode::If(
        Box::new(FnNode::Compare(
            Box::new(FnNode::Mul(Box::new(FnNode::X), Box::new(FnNode::Y))),
            CompareKind::GreaterThan,
            Box::new(FnNode::Number(0.0)),
        )),
        Box::new(FnNode::Triple(
            Box::new(FnNode::X),
            Box::new(FnNode::Y),
            Box::new(FnNode::Number(1.0)),
        )),
        Box::new(FnNode::Triple(
            Box::new(FnNode::Number(0.0)),
            Box::new(FnNode::Number(0.0)),
            Box::new(FnNode::Number(0.0)),
        )),
    );

    // Split4 node
    let split4_node = FnNode::If(
        Box::new(FnNode::Compare(
            Box::new(FnNode::Mul(Box::new(FnNode::X), Box::new(FnNode::Y))),
            CompareKind::GreaterThan,
            Box::new(FnNode::Number(0.0)),
        )),
        Box::new(FnNode::Triple(
            Box::new(FnNode::X),
            Box::new(FnNode::Y),
            Box::new(FnNode::Number(1.0)),
        )),
        Box::new(FnNode::Triple(
            Box::new(FnNode::Mod(
                Box::new(FnNode::Add(
                    Box::new(FnNode::X),
                    Box::new(FnNode::Number(1e-3)),
                )),
                Box::new(FnNode::Add(
                    Box::new(FnNode::Y),
                    Box::new(FnNode::Number(1e-3)),
                )),
            )),
            Box::new(FnNode::Mod(
                Box::new(FnNode::Add(
                    Box::new(FnNode::X),
                    Box::new(FnNode::Number(1e-3)),
                )),
                Box::new(FnNode::Add(
                    Box::new(FnNode::Y),
                    Box::new(FnNode::Number(1e-3)),
                )),
            )),
            Box::new(FnNode::Mod(
                Box::new(FnNode::Add(
                    Box::new(FnNode::X),
                    Box::new(FnNode::Number(1e-3)),
                )),
                Box::new(FnNode::Add(
                    Box::new(FnNode::Y),
                    Box::new(FnNode::Number(1e-3)),
                )),
            )),
        )),
    );

    let node = grayscale_node;
    node.node_render();
}

fn main() {
    use crate::grammar::{Grammar, Rule};
    use crate::node::FnNode;
    let e = 0;
    let a = 1;
    let c = 2;

    let mut grammar = Grammar::new();
    grammar.add_rule(Rule {
        branches: vec![(
            FnNode::Triple(
                Box::new(FnNode::Rule(0)),
                Box::new(FnNode::Rule(0)),
                Box::new(FnNode::Rule(0)),
            ),
            1.0,
        )],
    });
    grammar.add_rule(Rule {
        branches: vec![
            (FnNode::Random, 1.0 / 3.0),
            (FnNode::X, 1.0 / 3.0),
            (FnNode::Y, 1.0 / 3.0),
        ],
    });

    grammar.add_rule(Rule {
        branches: vec![
            (FnNode::Rule(a), 1.0 / 4.0),
            (
                FnNode::Add(Box::new(FnNode::Rule(c)), Box::new(FnNode::Rule(c))),
                3.0 / 8.0,
            ),
            (
                FnNode::Mul(Box::new(FnNode::Rule(c)), Box::new(FnNode::Rule(c))),
                3.0 / 8.0,
            ),
        ],
    });
    println!("{}", grammar);

    let func = grammar.gen_rule(0, 3);
    println!("{:?}", func);
}
