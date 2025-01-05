mod grammar;
mod node;
mod simple;

use crate::grammar::{Grammar, GrammarBranch};
use crate::node::{ArithmeticOp, FnNode /*CompareOp*/, UnaryOp};

fn gen_fn_from_grammar() -> Option<FnNode> {
    // let e = 0;
    let a = 1;
    let c = 2;

    let mut grammar = Grammar::new();
    let _ = grammar.add_rule(
        vec![
            GrammarBranch {
                node: FnNode::Triple(
                    Box::new(FnNode::Rule(c)),
                    Box::new(FnNode::Rule(c)),
                    Box::new(FnNode::Rule(c)),
                ),
                weight: 1,
            },
            GrammarBranch {
                node: FnNode::Triple(
                    Box::new(FnNode::Rule(a)),
                    Box::new(FnNode::Rule(c)),
                    Box::new(FnNode::Rule(a)),
                ),
                weight: 1,
            },
        ],
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
            GrammarBranch {
                node: FnNode::Unary(
                    UnaryOp::Sqrt,
                    Box::new(FnNode::Arithmetic(
                        Box::new(FnNode::Arithmetic(
                            Box::new(FnNode::X),
                            ArithmeticOp::Mul,
                            Box::new(FnNode::X),
                        )),
                        ArithmeticOp::Add,
                        Box::new(FnNode::Arithmetic(
                            Box::new(FnNode::Y),
                            ArithmeticOp::Mul,
                            Box::new(FnNode::Y),
                        )),
                    )),
                ),
                weight: 2,
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
                node: FnNode::Arithmetic(
                    Box::new(FnNode::Rule(c)),
                    ArithmeticOp::Add,
                    Box::new(FnNode::Rule(c)),
                ),
                weight: 3,
            },
            GrammarBranch {
                node: FnNode::Arithmetic(
                    Box::new(FnNode::Rule(c)),
                    ArithmeticOp::Mul,
                    Box::new(FnNode::Rule(c)),
                ),
                weight: 3,
            },
        ],
        "C",
    );
    println!("{}\n", grammar);

    grammar.gen_rule(0, 12)
}

fn main() -> Result<(), String> {
    let mut func = gen_fn_from_grammar().unwrap();
    func.optimize()?;
    println!("Function:");
    println!("{}", func);
    /*
    Ok(())
    */
    func.render()
}
