use crate::evm::bytecode::executor::execution::{Execution, FunctionFlow};
use crate::evm::bytecode::executor::stack::{Frame, StackFrame};
use std::fmt::Display;

pub fn print_flow(flow: &FunctionFlow, width: usize) {
    for exec in flow.execution_tree() {
        print_execution(exec, width);
    }
    let results = flow.result();
    if !results.is_empty() {
        write("return (", 4, false);
        for (i, var) in results.iter().enumerate() {
            print!("var_{}", var.index());
            if i != results.len() - 1 {
                print!(", ");
            }
        }
        println!(")");
    }
}

pub fn print_execution(exec: &Execution, width: usize) {
    match exec {
        Execution::SetVar(val, calc) => {
            write(format!("let var_{} = {{", val.index()), width, true);
            write_frame(calc, width + 5, true);
            write("}", width, true);
        }
        Execution::Calc(calc) => {
            write_frame(calc, width, true);
        }
        Execution::Branch {
            cnd,
            true_br,
            false_br,
        } => {
            write("IF (", width, false);
            write_frame(cnd, 0, false);
            write(")", 0, true);
            write("True:", width, true);
            print_flow(true_br, width + 5);
            write("False:", width, true);
            print_flow(false_br, width + 5);
            println!()
        }
        Execution::Abort(code) => {
            write(format!("abort({code})"), width, true);
        }
    }
}

fn write_frame(calc: &StackFrame, width: usize, new_line: bool) {
    match calc.frame().as_ref() {
        Frame::Val(val) => {
            write(val, width, new_line);
        }
        Frame::Param(param) => {
            write(format!("param_{param}"), width, new_line);
        }
        Frame::Bool(val) => {
            write(val, width, new_line);
        }
        Frame::SelfAddress => {
            write("address", width, new_line);
        }
        Frame::Mem(rf, val) => {
            write_frame(rf.as_ref(), width, false);
            print!(" = ");
            write_frame(val.as_ref(), width, new_line);
        }
        Frame::Calc(op, val) => {
            write_frame(val, width, new_line);
            write(op, width, new_line);
        }
        Frame::Calc2(op, a, b) => {
            write_frame(a, width, new_line);
            write_frame(b, width, new_line);
            write(op, width, new_line);
        }
        Frame::Abort(code) => {
            write(*code, width, new_line);
        }
    }
}

fn write<D: Display>(line: D, width: usize, new_line: bool) {
    print!("{:width$} {line}", " ");
    if new_line {
        println!();
    }
}
