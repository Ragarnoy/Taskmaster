use std::io::Error;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn main() -> Result<(), Error> {
    let term = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGHUP, Arc::clone(&term))?;
    println!("hi");
    while !term.load(Ordering::Relaxed) {
        // Do some time-limited stuff here
        // (if this could block forever, then there's no guarantee the signal will have any
        // effect).
        print!(".");
        std::thread::sleep(std::time::Duration::from_millis(100));
        std::io::stdout().flush()?;
    }
    println!("\nbye");
    term.store(false, Ordering::Relaxed);
    println!("or not");
    while !term.load(Ordering::Relaxed) {
        // Do some time-limited stuff here
        // (if this could block forever, then there's no guarantee the signal will have any
        // effect).
        print!(".");
        std::thread::sleep(std::time::Duration::from_millis(100));
        std::io::stdout().flush()?;
    }
    println!("\nbye");
    Ok(())
}
