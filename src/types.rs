use std::clone;

#[derive(Clone, Debug,PartialEq)]
pub enum Types{
    Int,
    Float,
    Bool
}

#[derive(Clone,Debug,Hash, Eq, PartialEq)]
pub enum Literal{
    Int(i64),
    //Float(f64),
    Bool(bool)
}

#[derive(Clone,Debug,PartialEq)]
pub enum Instruction{
    Const {dest: String, typ: Types, values: Literal},
    Add {dest: String, op1: String, op2: String},
    Mul {dest: String, op1: String, op2: String},
    Eq {dest: String, op1: String, op2: String},
    Jmp {label: String},
    Move {dest: String, src: String},
    Id {dest: String, src: String},
    Label {label: String},
    Br { cond: String, then_label: String, else_label: String},
    Ret {value: Option<String>},
    Print {value: String}

}


pub struct Block{
    pub label: String,
    pub instrs: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Function{
    pub name: String,
    pub instr: Vec<Instruction>
}

