use entrypoint::{handle_result, EntryPoint};

mod cliargs;
mod entrypoint;
mod run;
mod taskfile;
mod utils;

fn main() {
    let entrypoint = EntryPoint::new(None);
    match entrypoint {
        Ok(ep) => handle_result(ep.run()),
        Err(e) => handle_result(Err(e)),
    };
}
