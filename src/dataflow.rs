use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;

use crate::cfg::*;
//use crate::lvn::*;
use petgraph::Direction;
use petgraph::{graph::DiGraph, graph::NodeIndex};
use crate::lvn::get_dest;
use crate::types::*;

// Now to use for dataflow analysis

//fisrt we define reaching defintions

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub struct Definition{
    pub var: String,
    pub block: String,
    pub instr_index: usize,
}
pub struct ReachingDefintions{
    pub in_sets: HashMap<NodeIndex, HashSet<Definition>>,
    pub out_sets: HashMap<NodeIndex, HashSet<Definition>>
}



//first case reaching definition
pub fn reaching_definitions(cfg: &DiGraph<BasicBlock,()>) -> ReachingDefintions {

    let mut in_sets:HashMap<NodeIndex, HashSet<Definition>> = HashMap::new();
    let mut out_sets:HashMap<NodeIndex, HashSet<Definition>> = HashMap::new();

    // Compute GEN and KILL for each block
    let mut gen_sets: HashMap<NodeIndex, HashSet<Definition>> = HashMap::new();
    let mut kill_sets: HashMap<NodeIndex, HashSet<Definition>> = HashMap::new();

    for node_idx in cfg.node_indices(){
        let (gens,kills) = gen_and_kill(&cfg[node_idx]);
        gen_sets.insert(node_idx, gens.clone());
        kill_sets.insert(node_idx, kills);
        out_sets.insert(node_idx, gens);
    }

    let mut worklist: Vec<NodeIndex> = cfg.node_indices().collect();

    // while worklist is not empty we pop things out
    while let Some(b) =  worklist.pop(){
        
        // get predecessors
        let mut new_in = HashSet::new();
        for pred in cfg.neighbors_directed(b, petgraph::Direction::Incoming){
            new_in.extend(out_sets[&pred].iter().cloned());
        }

        let old_out = out_sets[&b].clone();
        let gens = &gen_sets[&b];
        let kills = &kill_sets[&b];
        //merge meanining gen[b] U (in[b] - kill[b])
        let mut new_out = gens.clone();
        for def in &new_in{
            if !kills.contains(def) {
                new_out.insert(def.clone());
            }
        }
        in_sets.insert(b, new_in);
        out_sets.insert(b, new_out.clone());

        if new_out != old_out {
            for succ in cfg.neighbors(b) {
                if !worklist.contains(&succ){
                    worklist.push(succ);
                }
            }
        }
    }
    

    ReachingDefintions { in_sets: in_sets, out_sets: out_sets }
}


//lets compute the gen and kill part of code
fn gen_and_kill(block: &BasicBlock) -> (HashSet<Definition>, HashSet<Definition>){

    let mut defs_by_var: HashMap<String, Definition> = HashMap::new();
    let mut kills = HashSet::new();

    for (i,instr) in block.instructions.iter().enumerate(){
        if let Some(dest) = get_dest(instr){
        let def = Definition {
            var: dest.clone(),
            block: block.name.clone(),
            instr_index: i,
        };
        // If variable was already defined, the old def is killed
        if let Some(old_def) = defs_by_var.insert(dest.clone(), def) {
            kills.insert(old_def);
            }
        }
    }
    let gens: HashSet<Definition> = defs_by_var.values().cloned().collect();

    (gens,kills)
}


// Abstract dataflow struct
pub enum GDirection {
    Forward,
    Backward,
}
pub trait AbstractDataflow {

    // this will be a Definition or String or Whatever we may need
    type Domain: Clone + PartialEq + Eq + Hash;

    fn direction() -> GDirection;
    /// Bottom element - initial/empty value
    fn bottom() -> HashSet<Self::Domain>;

    //merge combines either preds or succs depending on direction 
    fn merge(in_sets: Vec<HashSet<Self::Domain>>) -> HashSet<Self::Domain>;

    //transfer moving from out to in here its like the gen[b] U (in[b] - kill[b]) for example reaching defs
    fn transfer(block: &BasicBlock, in_set: HashSet<Self::Domain>) -> HashSet<Self::Domain>;
}

mod tests {
    use super::*;

    #[test]
fn test_reaching_definitions() {
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
    let rd = reaching_definitions(&cfg);

    println!("--- REACHING DEFINITIONS ANALYSIS ---\n");
    
    for node_idx in cfg.node_indices() {
        let block = &cfg[node_idx];
        println!("Block: {}", block.name);
        
        println!("  IN:");
        if let Some(in_set) = rd.in_sets.get(&node_idx) {
            if in_set.is_empty() {
                println!("    {{}}");
            } else {
                for def in in_set {
                    println!("    ({}, {}, instr {})", def.var, def.block, def.instr_index);
                }
            }
        }
        
        println!("  OUT:");
        if let Some(out_set) = rd.out_sets.get(&node_idx) {
            if out_set.is_empty() {
                println!("    {{}}");
            } else {
                for def in out_set {
                    println!("    ({}, {}, instr {})", def.var, def.block, def.instr_index);
                }
            }
        }
        println!();
    }
}

}