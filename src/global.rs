use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use crate::cfg::*;
use crate::dataflow::Definition;
use petgraph::Direction;
use petgraph::graph::Node;
use petgraph::{graph::DiGraph, graph::NodeIndex};
use crate::types::*;


pub fn find_dominators(cfg: &DiGraph<BasicBlock, ()>) -> HashMap<NodeIndex, HashSet<NodeIndex>> {
    let mut dom: HashMap<NodeIndex, HashSet<NodeIndex>> = HashMap::new();

    // Initialize: all nodes dominate all nodes
    let all_nodes: HashSet<NodeIndex> = cfg.node_indices().collect();
    for node in cfg.node_indices() {
        dom.insert(node, all_nodes.clone());
    }

    let entry = cfg.node_indices().next().unwrap();
    // for entry node we only insert itself
    dom.insert(entry, {
        let mut s = HashSet::new();
        s.insert(entry);
        s
    });


    loop {
        let old_dom = dom.clone();
        for vertex in cfg.node_indices(){
            let preds: Vec<NodeIndex> = cfg.neighbors_directed(vertex, petgraph::Direction::Incoming).collect();
            //let mut dom_pred: HashSet<NodeIndex> = HashSet::new();
            if !preds.is_empty(){
                let mut new_dom = dom[&preds[0]].clone(); 
                for pred in &preds[1..]{
                    new_dom = new_dom.intersection(&dom[pred]).copied().collect();
                }

                new_dom.insert(vertex);
                dom.insert(vertex, new_dom);

            }
        }
        if old_dom == dom {
            break;
        }
    }
    dom
}


pub fn build_dominator_tree(dom: &HashMap<NodeIndex, HashSet<NodeIndex>>) 
    -> HashMap<NodeIndex, Option<NodeIndex>> 
{
    let mut idom_tree: HashMap<NodeIndex, Option<NodeIndex>> = HashMap::new();
    
    for (&node, dominators) in dom {
        // Entry node has no immediate dominator
        if dominators.len() == 1 {
            idom_tree.insert(node, None);
            continue;
        }
        
        // Find all dominators except the node itself
        let mut candidates: Vec<NodeIndex> = Vec::new();
        for &d in dominators {
            if d != node {
                candidates.push(d);
            }
        }
        
        // idom is the candidate with the largest dom set
        let idom = candidates.iter()
            .max_by_key(|&&d| dom[&d].len())
            .copied();
        
        idom_tree.insert(node, idom);
    }
    
    idom_tree
}

fn find_dominance_frontier(
    cfg: &DiGraph<BasicBlock,()>, 
    dom: &HashMap<NodeIndex, HashSet<NodeIndex>>,
    idom: &HashMap<NodeIndex, Option<NodeIndex>>) -> HashMap<NodeIndex, HashSet<NodeIndex>> {
    let mut df : HashMap<NodeIndex, HashSet<NodeIndex>> = HashMap::new();

    for node in cfg.node_indices(){
        df.insert(node, HashSet::new()); 
    }
    for node in cfg.node_indices(){
        //ignore entry

        //
        let mut preds: Vec<NodeIndex> = cfg.neighbors_directed(node, Direction::Incoming).collect();

        if preds.len() >= 2 {

            for pred in preds {
                let mut runner = pred;

                while !dom[&node].contains(&runner) {
                    // here dominator set of pred of y doesnt dominate y, so y can be in its DF
                    df.entry(runner).or_insert_with(HashSet::new).insert(node);


                    // look at immediate dom of pred or runner i woould say
                    match idom[&runner]{
                        Some(parent) => runner = parent,
                        None => break,
                    }
                }

            }
        }
    }

    df
}

mod tests{
    use super::*;

    #[test]
    fn test_dominators() {
        let f = Function {
            name: "Main".to_string(),
            instr: vec![
                // ---- block0 ----
                Instruction::Const {
                    dest: "v0".to_string(),
                    typ: Types::Bool,
                    values: Literal::Bool(true),
                },
                Instruction::Br {
                    cond: "v0".to_string(),
                    then_label: "then_blk".to_string(),
                    else_label: "else_blk".to_string(),
                },
    
                // ---- then_blk (block1) ----
                Instruction::Label{label: "then_blk".to_string()},
                Instruction::Const {
                    dest: "v1".to_string(),
                    typ: Types::Int,
                    values: Literal::Int(10),
                },
                Instruction::Jmp {
                    label: "merge_blk".to_string(),
                },
    
                // ---- else_blk (block2) ----
                Instruction::Label{label:"else_blk".to_string()},
                Instruction::Const {
                    dest: "v2".to_string(),
                    typ: Types::Int,
                    values: Literal::Int(20),
                },
                Instruction::Jmp {
                    label: "merge_blk".to_string(),
                },
    
                // ---- merge_blk (block3) ----
                Instruction::Label{label: "merge_blk".to_string()},
                Instruction::Ret {
                    value: Some("v1".to_string()),
                },
            ],
        };
    
        let blocks = build_blocks(&f);
        let cfg = build_cfg(&blocks);
        let dom = find_dominators(&cfg);
    
        println!("--- DOMINATORS ---\n");
        
        for node_idx in cfg.node_indices() {
            let block = &cfg[node_idx];
            println!("Block: {}", block.name);
            println!("  Dominated by:");
            if let Some(doms) = dom.get(&node_idx) {
                let mut dom_names: Vec<String> = doms.iter()
                    .map(|d| cfg[*d].name.clone())
                    .collect();
                dom_names.sort();
                for name in dom_names {
                    println!("    {}", name);
                }
            }
            println!();
        }
    }


    #[test]
fn test_dominator_tree() {
    let f = Function {
        name: "Main".to_string(),
        instr: vec![
            Instruction::Const {
                dest: "v0".to_string(),
                typ: Types::Bool,
                values: Literal::Bool(true),
            },
            Instruction::Br {
                cond: "v0".to_string(),
                then_label: "then_blk".to_string(),
                else_label: "else_blk".to_string(),
            },
            Instruction::Label{label: "then_blk".to_string()},
            Instruction::Const {
                dest: "v1".to_string(),
                typ: Types::Int,
                values: Literal::Int(10),
            },
            Instruction::Jmp {
                label: "merge_blk".to_string(),
            },
            Instruction::Label{label:"else_blk".to_string()},
            Instruction::Const {
                dest: "v2".to_string(),
                typ: Types::Int,
                values: Literal::Int(20),
            },
            Instruction::Jmp {
                label: "merge_blk".to_string(),
            },
            Instruction::Label{label: "merge_blk".to_string()},
            Instruction::Ret {
                value: Some("v1".to_string()),
            },
        ],
    };

    let blocks = build_blocks(&f);
    let cfg = build_cfg(&blocks);
    let dom = find_dominators(&cfg);
    let idom = build_dominator_tree(&dom);

    println!("--- DOMINATOR TREE ---\n");
    
    for node_idx in cfg.node_indices() {
        let block = &cfg[node_idx];
        match idom.get(&node_idx) {
            Some(Some(parent_idx)) => {
                let parent = &cfg[*parent_idx];
                println!("{} → {}", parent.name, block.name);
            }
            Some(None) => {
                println!("{} (entry)", block.name);
            }
            None => {}
        }
    }
}

#[test]
fn test_dominance_frontier() {
    let f = Function {
        name: "Main".to_string(),
        instr: vec![
            // ---- block0 ----
            Instruction::Const {
                dest: "v0".to_string(),
                typ: Types::Bool,
                values: Literal::Bool(true),
            },
            Instruction::Br {
                cond: "v0".to_string(),
                then_label: "then_blk".to_string(),
                else_label: "else_blk".to_string(),
            },

            // ---- then_blk ----
            Instruction::Label{label: "then_blk".to_string()},
            Instruction::Const {
                dest: "v1".to_string(),
                typ: Types::Int,
                values: Literal::Int(10),
            },
            Instruction::Jmp {
                label: "merge_blk".to_string(),
            },

            // ---- else_blk ----
            Instruction::Label{label:"else_blk".to_string()},
            Instruction::Const {
                dest: "v2".to_string(),
                typ: Types::Int,
                values: Literal::Int(20),
            },
            Instruction::Jmp {
                label: "merge_blk".to_string(),
            },

            // ---- merge_blk ----
            Instruction::Label{label: "merge_blk".to_string()},
            Instruction::Ret {
                value: Some("v1".to_string()),
            },
        ],
    };

    let blocks = build_blocks(&f);
    let cfg = build_cfg(&blocks);
    let dom = find_dominators(&cfg);
    let idom = build_dominator_tree(&dom);
    let df = find_dominance_frontier(&cfg, &dom, &idom);

    println!("--- DOMINANCE FRONTIER ---\n");
    
    for node_idx in cfg.node_indices() {
        let block = &cfg[node_idx];
        println!("DF({}): ", block.name);
        if let Some(frontier) = df.get(&node_idx) {
            if frontier.is_empty() {
                println!("  {{}}");
            } else {
                let mut names: Vec<String> = frontier.iter()
                    .map(|n| cfg[*n].name.clone())
                    .collect();
                names.sort();
                for name in names {
                    println!("  {}", name);
                }
            }
        }
        println!();
    }
}

}