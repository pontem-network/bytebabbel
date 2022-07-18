use crate::evm::bytecode::executor::execution::{Execution, FunctionFlow};
use crate::evm::bytecode::executor::stack::{Frame, StackFrame};
use std::fmt::Display;

pub fn print_flow(flow: &FunctionFlow, width: usize) {
    for exec in flow.execution_tree() {
        print_execution(exec, width);
    }
    let results = flow.result();
    if !results.is_empty() {
        let mut log = format!("{:4} return (", " ");
        for (i, var) in results.iter().enumerate() {
            log += format!("var_{}", var.index()).as_str();
            if i != results.len() - 1 {
                log += format!(", ").as_str();
            }
        }
        log += ")";
        logs::trace!("{log}");
    }
}

pub fn print_execution(exec: &Execution, width: usize) {
    match exec {
        Execution::SetVar(val, calc) => {
            write(format!("let var_{} = {{", val.index()), width);
            write_frame(calc, width + 5);
            write("}}", width);
        }
        Execution::Calc(calc) => {
            write_frame(calc, width);
        }
        Execution::Branch { true_br, false_br } => {
            write("IF", width);
            write("True:", width);
            print_flow(true_br, width + 5);
            write("False:", width);
            print_flow(false_br, width + 5);
        }
        Execution::Abort(code) => {
            write(format!("abort({code})"), width);
        }
    }
}

fn write_frame(calc: &StackFrame, width: usize) {
    match calc.frame().as_ref() {
        Frame::Val(val) => {
            write(val, width);
        }
        Frame::Param(param) => {
            write(format!("param_{param}"), width);
        }
        Frame::Bool(val) => {
            write(val, width);
        }
        Frame::SelfAddress => {
            write("address", width);
        }
        Frame::Mem(rf, val) => {
            write_frame(rf.as_ref(), width);
            logs::trace!(" = ");
            write_frame(val.as_ref(), width);
        }
        Frame::Calc(op, val) => {
            write_frame(val, width);
            write(op, width);
        }
        Frame::Calc2(op, a, b) => {
            write_frame(a, width);
            write_frame(b, width);
            write(op, width);
        }
        Frame::Abort(code) => {
            write(*code, width);
        }
    }
}

fn write<D: Display>(line: D, width: usize) {
    logs::trace!("{:width$} {line}", " ");
}
