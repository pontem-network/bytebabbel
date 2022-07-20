use crate::evm::bytecode::executor::execution::{Execution, FunctionFlow};
use crate::evm::bytecode::executor::stack::{Frame, StackFrame};
use std::fmt::Display;

pub fn output_flow(flow: &FunctionFlow, width: usize) -> String {
    let mut output = String::new();
    for exec in flow.execution_tree() {
        output += output_execution(exec, width).as_str();
    }
    let results = flow.result();
    if !results.is_empty() {
        output += format!("{:4} return (", " ").as_str();
        for (i, var) in results.iter().enumerate() {
            output += format!("var_{}", var.index()).as_str();
            if i != results.len() - 1 {
                output += ", ";
            }
        }
        output += ")";
    }
    output
}

pub fn output_execution(exec: &Execution, width: usize) -> String {
    let mut output = String::new();
    match exec {
        Execution::SetVar(val, calc) => {
            output += spaces_with_ln(format!("let var_{} = {{", val.index()), width).as_str();
            output += write_frame(calc, width + 5).as_str();
            output += spaces_with_ln("}", width).as_str();
        }
        Execution::Calc(calc) => {
            write_frame(calc, width);
        }
        Execution::Branch { cnd, true_br, false_br } => {
            spaces_with("IF(", width);
            spaces_with(write_frame(cnd, 0).as_str(), 0);
            spaces_with_ln("True:", 0);
            spaces_with_ln("True:", width);
            output_flow(true_br, width + 5);
            spaces_with_ln("False:", width);
            output_flow(false_br, width + 5);
        }
        Execution::Abort(code) => {
            spaces_with_ln(format!("abort({code})"), width);
        }
    }
    output
}

fn write_frame(calc: &StackFrame, width: usize) -> String {
    let mut output;
    match calc.frame().as_ref() {
        Frame::Val(val) => {
            output = spaces_with_ln(val, width);
        }
        Frame::Param(param) => {
            output = spaces_with_ln(format!("param_{param}"), width);
        }
        Frame::Bool(val) => {
            output = spaces_with_ln(val, width);
        }
        Frame::SelfAddress => {
            output = spaces_with_ln("address", width);
        }
        Frame::Mem(rf, val) => {
            output = write_frame(rf.as_ref(), width);
            output += " = ";
            output += write_frame(val.as_ref(), width).as_str();
        }
        Frame::Calc(op, val) => {
            output = write_frame(val, width);
            output += spaces_with_ln(op, width).as_str();
        }
        Frame::Calc2(op, a, b) => {
            output = write_frame(a, width);
            output += write_frame(b, width).as_str();
            output += spaces_with_ln(op, width).as_str();
        }
        Frame::Abort(code) => {
            output = spaces_with_ln(*code, width);
        }
    }
    output
}

fn spaces_with_ln<D: Display>(line: D, width: usize) -> String {
    format!("{:width$} {line}\n", " ")
}
