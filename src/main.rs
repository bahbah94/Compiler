pub mod types;
pub mod cfg;
pub mod lvn;
use types::*;
use cfg::*;
use lvn::*;


fn main() {
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
    let cFG = create_cfg(&blocks);

    println!("CFG IS {:?}", cFG);


    
}
