use crate::Ir;
use anyhow::Error;
use log::log_enabled;
use log::Level;
use std::fmt::Write;

pub fn print_ir(ir: &Ir) {
    if log_enabled!(Level::Trace) {
        let mut buf = String::new();
        if let Err(err) = print_buf(ir, &mut buf, 0) {
            log::error!("Failed to print ir: {}", err);
        }
        log::trace!("IR:\n{}", buf);
    }
}

fn print_buf(ir: &Ir, buf: &mut String, width: usize) -> Result<(), Error> {
    writeln!(buf, "=================================================================================")?;
    for inst in &ir.instructions {
        write!(buf, "{:width$} {:?}", " ", inst)?;
    }
    writeln!(buf, "=================================================================================")?;
    Ok(())
}
