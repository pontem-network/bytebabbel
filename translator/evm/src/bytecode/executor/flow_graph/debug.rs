use crate::bytecode::executor::flow_graph::flow::Flow;
use anyhow::Error;
use log::log_enabled;
use log::Level;
use std::fmt::Write;

pub fn log_flow(flow: &Flow) {
    if log_enabled!(Level::Trace) {
        let mut s = String::new();
        if let Err(err) = print_flow(&mut s, flow, 0) {
            log::warn!("Failed to print flow {}", err);
        } else {
            log::info!("\n{}", s);
        }
    }
}

fn print_flows<W: Write>(buf: &mut W, vec: &[Flow], width: usize) -> Result<(), Error> {
    for flow in vec {
        print_flow(buf, flow, width)?;
    }
    Ok(())
}

fn print_flow<W: Write>(buf: &mut W, flow: &Flow, width: usize) -> Result<(), Error> {
    match flow {
        Flow::Block(seq) => {
            writeln!(buf, "{:width$}0x{}", " ", seq)?;
        }
        Flow::Loop(loop_) => {
            writeln!(buf, "{:width$}loop {{", " ")?;
            print_flow(buf, &loop_.loop_br, width + 4)?;
            writeln!(buf, "{:width$}}}", " ")?;
        }
        Flow::IF(if_) => {
            writeln!(buf, "{:width$}if ({}) {{", " ", if_.jmp.block)?;
            print_flow(buf, &if_.true_br, width + 4)?;
            writeln!(buf, "{:width$}}} else {{", " ")?;
            print_flow(buf, &if_.false_br, width + 4)?;
            writeln!(buf, "{:width$}}}", " ")?;
        }
        Flow::Sequence(flow) => {
            print_flows(buf, flow, width)?;
        }
    }
    Ok(())
}
