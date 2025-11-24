use std::{collections::HashMap, hash::Hash};
use petgraph::{graph::DiGraph, graph::NodeIndex};


use crate::types::*;

pub fn build_blocks(f: &Function ) -> Vec<Vec<Instruction>>{
    let mut blocks = Vec::new();
    let mut current = Vec::new();

    for instru in &f.instr {

        current.push(instru.clone());

        if is_terminator(instru){
            blocks.push(current);
            current = Vec::new();
        } 
    }
    if !current.is_empty() {
        blocks.push(current);
    }
    blocks

}

#[derive(Clone,Debug)]
pub struct BasicBlock {
    pub name: String,
    pub instructions: Vec<Instruction>,
}


pub fn build_cfg(blocks: &Vec<Vec<Instruction>>) -> DiGraph<BasicBlock, ()> {
    let mut graph = DiGraph::new();
    let mut block_to_node: HashMap<String, NodeIndex> = HashMap::new();

    //name the blocks first
    for (i,block) in blocks.iter().enumerate() {
        let name = if i == 0{
            "block0".to_string()
        }else {
            if let Instruction::Label { label } = &block[0] {
                label.clone()
            } else {
                format!("block{}",i)
            }
        };
        let block = BasicBlock {
            name: name.clone(),
            instructions: block.clone()
        };

        let node = graph.add_node(block.clone());
        block_to_node.insert(name, node);
     }

     // Now to add the edge
     for (i,block) in blocks.iter().enumerate(){
        let block_name = get_block_name(i, block);
        let from = block_to_node[&block_name];
        let last = block.last().unwrap();

        let successors = match last {
            Instruction::Jmp { label } => vec![label.clone()],
            Instruction::Br { then_label, else_label, .. } => vec![then_label.clone(), else_label.clone()],
            Instruction::Ret { .. } => vec![],
            _ => {
                if i + 1 < blocks.len() {
                    vec![format!("block{}", i + 1)]
                } else {
                    vec![]
                }
            }
        };

        for succ in successors{
            if let Some(&to) = block_to_node.get(&succ){
                graph.add_edge(from, to, ());
            }
        }
     }

     graph
}

fn get_block_name(i: usize, block: &Vec<Instruction>) -> String {
    if i == 0 {
        "block0".to_string()
    } else {
        if let Instruction::Label { label } = &block[0] {
            label.clone()
        } else {
            format!("block{}", i)
        }
    }
}




fn is_terminator(i: &Instruction) -> bool {
    matches!(i, Instruction::Br {..} | Instruction::Jmp{..} | Instruction::Ret{..})
}

#[cfg(test)]
mod tests{
    use petgraph::Direction;

    use super::*;

    #[test]
    fn test_cfg_graph() {
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

        println!("--- CFG NODES ---");
        for node_idx in cfg.node_indices() {
            let block = &cfg[node_idx];
            println!("Node: {} (name: {})", node_idx.index(), block.name);
        }

        println!("\n--- SUCCESSORS ---");
        for node_idx in cfg.node_indices() {
            let block = &cfg[node_idx];
            let successors: Vec<_> = cfg.neighbors(node_idx).collect();
            let succ_names: Vec<String> = successors.iter()
                .map(|&succ_idx| cfg[succ_idx].name.clone())
                .collect();
            println!("{} -> {:?}", block.name, succ_names);
        }

        println!("\n--- PREDECESSORS ---");
        for node_idx in cfg.node_indices() {
            let block = &cfg[node_idx];
            let predecessors: Vec<_> = cfg.neighbors_directed(node_idx, Direction::Incoming).collect();
            let pred_names: Vec<String> = predecessors.iter()
                .map(|&pred_idx| cfg[pred_idx].name.clone())
                .collect();
            println!("{} <- {:?}", block.name, pred_names);
        }

        // Assertions
        assert_eq!(cfg.node_count(), 4);
        assert_eq!(cfg.edge_count(), 4);  // block0->then, block0->else, then->merge, else->merge, no return edge
    }

}
