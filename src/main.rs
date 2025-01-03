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

    let node = split4_node;
    node.node_render();
}

use crate::node::FnNode;
fn gen_fn_from_grammar() -> Option<FnNode> {
    use crate::grammar::{Grammar, GrammarBranch};
    let e = 0;
    let a = 1;
    let c = 2;

    let mut grammar = Grammar::new();
    let _ = grammar.add_rule(
        vec![GrammarBranch {
            node: FnNode::Triple(
                Box::new(FnNode::Rule(c)),
                Box::new(FnNode::Rule(c)),
                Box::new(FnNode::Rule(c)),
            ),
            weight: 1,
        }],
        "E",
    );
    let _ = grammar.add_rule(
        vec![
            GrammarBranch {
                node: FnNode::Random,
                weight: 1,
            },
            GrammarBranch {
                node: FnNode::X,
                weight: 1,
            },
            GrammarBranch {
                node: FnNode::Y,
                weight: 1,
            },
            // GrammarBranch {
            //     node: FnNode::T,
            //     weight: 1,
            // },
        ],
        "A",
    );
    let _ = grammar.add_rule(
        vec![
            GrammarBranch {
                node: FnNode::Rule(a),
                weight: 2,
            },
            GrammarBranch {
                node: FnNode::Add(Box::new(FnNode::Rule(c)), Box::new(FnNode::Rule(c))),
                weight: 3,
            },
            GrammarBranch {
                node: FnNode::Mul(Box::new(FnNode::Rule(c)), Box::new(FnNode::Rule(c))),
                weight: 3,
            },
        ],
        "C",
    );
    println!("{}", grammar);

    grammar.gen_rule(0, 12)
}

fn main() {
    let func = gen_fn_from_grammar().unwrap();
    println!("function:");
    println!("{}", func);
    func.node_render();
}
