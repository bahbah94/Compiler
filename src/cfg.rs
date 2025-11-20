use std::{collections::HashMap, hash::Hash};
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


pub fn create_cfg(blocks: &Vec<Vec<Instruction>>) -> HashMap<String, Vec<String>>{
    let mut cfg = HashMap::new();

    for (i, block) in blocks.iter().enumerate(){
        let name = format!("block{}",i);
        let last = block.last().unwrap();

        let succ = match last {
            Instruction::Jmp {label} => vec![label.clone()],
            Instruction::Br { cond: _cond, then_label, else_label } => vec![then_label.clone(), else_label.clone()],
            Instruction::Ret { .. } => vec![],
            _ => {
                if i +1 < blocks.len(){
                    vec![format!("block{}", i + 1)]
                } else {
                    vec![]
                }
            }
        };

        cfg.insert(name,succ);

    }
    cfg
}


fn is_terminator(i: &Instruction) -> bool {
    matches!(i, Instruction::Br {..} | Instruction::Jmp{..} | Instruction::Ret{..})
}