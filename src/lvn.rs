use std::collections::{HashMap, HashSet};

use crate::types::*;

#[derive(Eq, Hash, PartialEq,Clone)]
pub enum ExprKey {
    Const(Literal),
    Id(usize),
    Add(usize, usize),
    Mul(usize, usize),
    Eq(usize, usize),
    Move(usize),
}

pub fn lvn(block: &Vec<Instruction>) -> Vec<Instruction> {
    let mut var2num: HashMap<String, usize> = HashMap::new();
    let mut table: HashMap<ExprKey, usize> = HashMap::new();
    let mut canon_var: HashMap<usize, String> = HashMap::new();
    let mut num2instr: HashMap<usize, Instruction> = HashMap::new();
    let mut expr_for_num: HashMap<usize, ExprKey> = HashMap::new(); // to keep track of IDs ie number/index --> exprKeys
    let mut new_block = Vec::new();

    for instr in block {
        if let Some(expr_key) = get_var(instr, &var2num,&expr_for_num) {
            let dest_opt = get_dest(instr);
    
            if let Some(&num) = table.get(&expr_key) {
                if let Some(dest) = dest_opt {
                    var2num.insert(dest.clone(), num);
                    let canon = canon_var.get(&num).unwrap().clone();
                    if let Some(Instruction::Const { typ, values, .. }) = num2instr.get(&num){
                        let new_instr = Instruction::Const { 
                            dest: dest.to_string(), 
                            typ: typ.clone(), 
                            values: values.clone()
                         };
                        new_block.push(new_instr);
                    } else {
                        let new_instr = Instruction::Id {
                            dest: dest.clone(),
                            src: canon,
                        };
                        new_block.push(new_instr);
                    }
                } else {
                    // no destination, just push original
                    let canonical_instr = canonicalize_operands(instr, &var2num, &canon_var);
                    new_block.push(canonical_instr.clone());
                }
            } else {
                let idx = table.len() + 1;
                table.insert(expr_key.clone(), idx);
                expr_for_num.insert(idx,expr_key);
                num2instr.insert(idx,instr.clone());
    
                if let Some(dest) = dest_opt {
                    var2num.insert(dest.clone(), idx);
                    canon_var.insert(idx, dest.clone());
                }
                
                let canonical_instr = canonicalize_operands(instr, &var2num, &canon_var);
                new_block.push(canonical_instr);
            }
        } else {
            // expr_key is None, just push instruction
            let canonical_instr = canonicalize_operands(instr, &var2num, &canon_var);
            new_block.push(canonical_instr);
        }
    }
    new_block   
}

pub fn constant_fold(block: &Vec<Instruction>) -> Vec<Instruction> {
    let mut const_values: HashMap<String, Literal> = HashMap::new();
    let mut new_block = Vec::new();

    for instr in block {
        // Track constants
        if let Instruction::Const { dest, values, .. } = instr {
            const_values.insert(dest.clone(), values.clone());
            new_block.push(instr.clone());
            continue;
        }

        // Try to fold operations
        let folded = match instr {
            Instruction::Add { dest, op1, op2, .. } => {
                if let (Some(Literal::Int(v1)), Some(Literal::Int(v2))) = 
                    (const_values.get(op1), const_values.get(op2)) {
                    Some(Instruction::Const {
                        dest: dest.clone(),
                        typ: Types::Int,
                        values: Literal::Int(v1 + v2),
                    })
                } else {
                    None
                }
            }
            Instruction::Mul { dest, op1, op2, .. } => {
                if let (Some(Literal::Int(v1)), Some(Literal::Int(v2))) = 
                    (const_values.get(op1), const_values.get(op2)) {
                    Some(Instruction::Const {
                        dest: dest.clone(),
                        typ: Types::Int,
                        values: Literal::Int(v1 * v2),
                    })
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some(folded_instr) = folded {
            if let Instruction::Const { dest, values, .. } = &folded_instr {
                const_values.insert(dest.clone(), values.clone());
            }
            new_block.push(folded_instr);
        } else {
            new_block.push(instr.clone());
        }
    }

    new_block
}

fn canonicalize_operands(instr: &Instruction, var2num: &HashMap<String, usize>, canon_var: &HashMap<usize, String>) -> Instruction {
    match instr {
        Instruction::Add { dest, op1, op2 } => {
            let canon_op1 = get_canonical(op1, var2num, canon_var).unwrap_or(op1.clone());
            let canon_op2 = get_canonical(op2, var2num, canon_var).unwrap_or(op2.clone());
            Instruction::Add {
                dest: dest.clone(),
                op1: canon_op1,
                op2: canon_op2,
            }
        }
        Instruction::Mul { dest, op1, op2 } => {
            let canon_op1 = get_canonical(op1, var2num, canon_var).unwrap_or(op1.clone());
            let canon_op2 = get_canonical(op2, var2num, canon_var).unwrap_or(op2.clone());
            Instruction::Mul {
                dest: dest.clone(),
                op1: canon_op1,
                op2: canon_op2,
            }
        }
        Instruction::Eq { dest, op1, op2 } => {
            let canon_op1 = get_canonical(op1, var2num, canon_var).unwrap_or(op1.clone());
            let canon_op2 = get_canonical(op2, var2num, canon_var).unwrap_or(op2.clone());
            Instruction::Eq {
                dest: dest.clone(),
                op1: canon_op1,
                op2: canon_op2,
            }
        }
        Instruction::Move { dest, src } => {
            let canon_src = get_canonical(src, var2num, canon_var).unwrap_or(src.clone());
            Instruction::Move {
                dest: dest.clone(),
                src: canon_src,
            }
        }
        Instruction::Id { dest, src } => {
            let canon_src = get_canonical(src, var2num, canon_var).unwrap_or(src.clone());
            Instruction::Id {
                dest: dest.clone(),
                src: canon_src,
            }
        }
        Instruction::Print { value } => {
            let canon_value = get_canonical(value, var2num, canon_var).unwrap_or(value.clone());
            Instruction::Print {
                value: canon_value,
            }
        }
        // Other instructions (Const, Jmp, etc.) have no operands to canonicalize
        other => other.clone(),
    }
}

// helper for canonical 
fn get_canonical(var: &String, var2num: &HashMap<String, usize>, canon_var: &HashMap<usize, String>) -> Option<String> {
    var2num.get(var).and_then(|&num| canon_var.get(&num).cloned())
}

// helper 
pub fn get_dest(instr: &Instruction) -> Option<&String> {
    match instr {
        Instruction::Id { dest, src } => Some(dest),
        Instruction::Const { dest, .. } => Some(dest),
        Instruction::Add { dest, .. } => Some(dest),
        Instruction::Mul { dest, .. } => Some(dest),
        Instruction::Eq { dest, .. } => Some(dest),
        Instruction::Move { dest, .. } => Some(dest),
        Instruction::Jmp { .. } => None, // no dest here
        _ => None
    }
}



fn get_var(instr: &Instruction, var2num: &HashMap<String, usize>, expr_for_num: &HashMap<usize,ExprKey>) -> Option<ExprKey> {
    match instr {
        Instruction::Const { values, .. } => Some(ExprKey::Const(values.clone())),

        Instruction::Id { src, .. } | Instruction::Move { src, .. } => {
            if let Some(&idx) = var2num.get(src) {
                if let Some(expr_key) = expr_for_num.get(&idx) {
                    Some(expr_key.clone())
                } else {
                    Some(ExprKey::Id(idx))
                }
            } else {
                None // src not yet mapped
            }
        }

        Instruction::Add { op1, op2, .. } => {
            if let (Some(&i1), Some(&i2)) = (var2num.get(op1), var2num.get(op2)) {
                let mut idxs = vec![i1, i2];
                idxs.sort();
                Some(ExprKey::Add(idxs[0], idxs[1]))
            } else {
                None
            }
        }

        Instruction::Mul { op1, op2, .. } => {
            if let (Some(&i1), Some(&i2)) = (var2num.get(op1), var2num.get(op2)) {
                let mut idxs = vec![i1, i2];
                idxs.sort();
                Some(ExprKey::Mul(idxs[0], idxs[1]))
            } else {
                None
            }
        }

        Instruction::Eq { op1, op2, .. } => {
            if let (Some(&i1), Some(&i2)) = (var2num.get(op1), var2num.get(op2)) {
                Some(ExprKey::Eq(i1, i2))
            } else {
                None
            }
        }

        Instruction::Print { value } => {
            var2num.get(value).map(|&idx| ExprKey::Id(idx))
        }

        _ => None,
    }
}

fn get_used_var(instr : &Instruction) -> Vec<String> {
    match instr {
        Instruction::Add { dest, op1, op2 } => vec![op1.clone(), op2.clone()],
        Instruction::Const { dest, typ, values } => vec![],
        Instruction::Eq { dest, op1, op2 } => vec![op1.clone(), op2.clone()],
        Instruction::Mul { dest, op1, op2 } => vec![op1.clone(), op2.clone()],
        Instruction::Move { dest, src } => vec![src.clone()],
        Instruction::Id { dest, src } => vec![src.clone()],
        Instruction::Print { value } => vec![value.clone()],
        Instruction::Ret { value } => {
            if let Some(v) = value {
                vec![v.clone()]
            } else {
                vec![]
            }
        }
        Instruction::Br { cond, .. } => vec![cond.clone()],
        _ => vec![] 
    }
}

fn dead_elimination_unused(block: &Vec<Instruction>) -> Vec<Instruction> {
    let mut used_vars: HashSet<String> = HashSet::new();
    
    // Collect all variables that are USED
    for instr in block {
        let used = get_used_var(instr);
        for var in used {
            used_vars.insert(var);
        }
    }
    
    // Keep only instructions whose destination is used
    let mut new_block = Vec::new();
    for instr in block {
        if let Some(dest) = get_dest(instr) {
            if used_vars.contains(dest) {
                new_block.push(instr.clone());
            }
        } else {
            new_block.push(instr.clone());
        }
    }
    
    new_block
}


// put the iterative version of above func 
fn dce_combined(block: &Vec<Instruction>) -> Vec<Instruction> {
    let mut current_block = block.clone();
    
    let pass1 = dead_elimination_unused(&current_block);
    let pass2 = dead_elimination_redefined(&pass1);
    current_block
    
}
fn dead_elimination_redefined(block: &Vec<Instruction>) -> Vec<Instruction> {
    let mut last_def: HashMap<String, usize> = HashMap::new();  // var -> index of last def
    let mut used_instrs: HashSet<usize> = HashSet::new();  // indices of used definitions

    for (i, instr) in block.iter().enumerate() {
        let used = get_used_var(instr);

        for var in &used {
            // Mark the LAST definition of this variable as used
            if let Some(&def_idx) = last_def.get(var) {
                used_instrs.insert(def_idx);
            }
            last_def.remove(var);
        }

        if let Some(dest) = get_dest(instr) {
            last_def.remove(dest);
            last_def.insert(dest.to_string(), i);  // Store index, not instruction
        }
    }

    // Also keep all final definitions (last_def at end of loop)
    for (_, &idx) in last_def.iter() {
        used_instrs.insert(idx);
    }

    let mut new_block = Vec::new();
    for (i, instr) in block.iter().enumerate() {
        if used_instrs.contains(&i) || get_dest(instr).is_none() {
            new_block.push(instr.clone());
        }
    }

    new_block
}

fn final_local_opt(block: &Vec<Instruction>) -> Vec<Instruction>{
    let mut current_block = lvn(&block);
    current_block = constant_fold(&current_block);

    loop {

        let before = format!("{:?}", current_block);

        current_block = dead_elimination_unused(&current_block);
        current_block = dead_elimination_redefined(&current_block);

        let after = format!("{:?}", current_block);

        if before == after {
            break;
        }
    }
    current_block
}


// =====================================
//             TESTS
// =====================================
#[cfg(test)]
mod tests {
    use super::*;

    fn print_block(label: &str, block: &Vec<Instruction>) {
        println!("--- {} ---", label);
        for instr in block {
            println!("{:?}", instr);
        }
        println!("-------------------\n");
    }

    #[test]
    fn test_cse_and_copyprop() {
        let block = vec![
            Instruction::Const { dest: "a".into(), typ: Types::Int, values: Literal::Int(1) },
            Instruction::Const { dest: "b".into(), typ: Types::Int, values: Literal::Int(2) },
            Instruction::Add { dest: "sum1".into(), op1: "a".into(), op2: "b".into() },
            Instruction::Add { dest: "sum2".into(), op1: "a".into(), op2: "b".into() }, // CSE -> Id sum1
            Instruction::Mul { dest: "prod".into(), op1: "sum1".into(), op2: "sum2".into() },
            Instruction::Print { value: "prod".into() },
        ];

        print_block("Original block", &block);

        let lvn_block = lvn(&block);
        let dce_elim = dead_elimination_unused(&lvn_block);
        print_block("After LVN (CSE + copy propagation) + dead code elim simple", &dce_elim);
    }

    #[test]
    fn test_copy_propagation_only() {
        let block = vec![
            Instruction::Const { dest: "a".into(), typ: Types::Int, values: Literal::Int(10) },
            Instruction::Id { dest: "b".into(), src: "a".into() },
            Instruction::Id { dest: "c".into(), src: "b".into() },
            Instruction::Id { dest: "d".into(), src: "c".into() },
            Instruction::Print { value: "d".into() }
        ];
    
        print_block("Original block", &block);
    
        let lvn_block = lvn(&block);
        //let dce = dead_elimination_unused(&lvn_block);
        print_block("After LVN(For copy prop) + constant folding", &lvn_block);
    }

    #[test]
    fn test_constant_folding(){
        let block = vec![
            Instruction::Const { dest: "a".into(), typ: Types::Int, values: Literal::Int(4) },
            Instruction::Const { dest: "b".into(), typ: Types::Int, values: Literal::Int(2) },
            Instruction::Add { dest: "sum1".into(), op1: "a".into(), op2: "b".into() },
            Instruction:: Add { dest: "sum2".into(), op1: "b".into(), op2: "a".into() },
            Instruction::Mul { dest: "prod1".into(), op1: "sum1".into(), op2: "sum2".into() },
            Instruction::Const { dest: "sum1".into(), typ: Types::Int, values: Literal::Int(0) },
            Instruction::Const { dest: "sum2".into(), typ: Types::Int, values: Literal::Int(0) },
            Instruction::Add { dest: "sum3".into(), op1: "a".into(), op2: "b".into() },
            Instruction::Mul { dest: "prod2".into(), op1: "sum3".into(), op2: "sum3".into() },
            Instruction::Print { value: "prod2".into() }
        ];

        print_block("Orginal Block is :", &block);
        let lvn_block = lvn(&block);
        let fold_block = constant_fold(&lvn_block);
        let dce_block = dead_elimination_unused(&fold_block);
        let redefined_block = dead_elimination_redefined(&dce_block);
        let final_block = dead_elimination_unused(&redefined_block);
        print_block("After LVN + constantFolding + copy prop + cse + dce + redefined", &final_block);

        print_block("Using Final optimization function", &final_local_opt(&block));
    }

    #[test]
fn run_dead_code_elimination_only(){
    let block = vec![
        Instruction::Const { dest: "a".into(), typ: Types::Int, values: Literal::Int(5) },
        Instruction::Const { dest: "b".into(), typ: Types::Int, values: Literal::Int(10) },
        Instruction::Const {dest: "b".into(), typ: Types::Int, values: Literal::Int(6)},
        Instruction::Add { dest: "c".into(), op1: "a".into(), op2: "b".into() },
        Instruction::Const { dest: "a".into(), typ: Types::Int, values: Literal::Int(7) }, // DEAD: a is redefined before use
        Instruction::Mul { dest: "d".into(), op1: "c".into(), op2: "a".into() },
        Instruction::Const { dest: "e".into(), typ: Types::Int, values: Literal::Int(100) }, // DEAD: e is never used
        Instruction::Print { value: "d".into() },
    ];

    print_block("Original Block", &block);

    let pass1 = dead_elimination_unused(&block);
    print_block("After dead_elimination_unused", &pass1);

    let pass2 = dead_elimination_redefined(&pass1);
    print_block("After dead_elimination_redefined", &pass2);

    //let final_block = dead_elimination_iterative(&block);
    //print_block("After dead_elimination_iterative (both passes combined)", &final_block);
}

 #[test]
 fn run_only_redefined(){
    let block = vec![
        Instruction::Const { dest: "a".into(), typ: Types::Int, values: Literal::Int(5) },
        Instruction::Const { dest: "a".into(), typ: Types::Int, values: Literal::Int(10) },
        Instruction::Const {dest: "a".into(), typ: Types::Int, values: Literal::Int(50)},
        Instruction::Print { value: "a".into() }
    ];

    print_block("Original Block", &block);

    let pass1 = dead_elimination_redefined(&block);
    print_block("After dead_elimination_refined", &pass1);


 }

 #[test]
fn test_dead_elimination_iterative_comprehensive() {
    let block = vec![
        Instruction::Const { dest: "a".into(), typ: Types::Int, values: Literal::Int(5) },
        Instruction::Const { dest: "b".into(), typ: Types::Int, values: Literal::Int(10) },
        Instruction::Const { dest: "b".into(), typ: Types::Int, values: Literal::Int(6) },
        Instruction::Add { dest: "c".into(), op1: "a".into(), op2: "b".into() },
        Instruction::Const { dest: "a".into(), typ: Types::Int, values: Literal::Int(7) },
        Instruction::Mul { dest: "d".into(), op1: "c".into(), op2: "a".into() },
        Instruction::Const { dest: "e".into(), typ: Types::Int, values: Literal::Int(100) }, // DEAD: never used
        Instruction::Const { dest: "f".into(), typ: Types::Int, values: Literal::Int(50) },  // DEAD: never used
        Instruction::Const { dest: "f".into(), typ: Types::Int, values: Literal::Int(75) },  // DEAD: overwritten before use
        Instruction::Print { value: "d".into() },
    ];

    print_block("Original Block", &block);

    let final_block = dead_elimination_unused(&block);
    print_block("After iterative DCE (combined)", &final_block);

    // Expected: removes e, both f definitions, and b=10
    // Keeps: a=5, b=6, c=a+b, a=7, d=c*a, print d
    //assert_eq!(final_block.len(), 6);
}
}
