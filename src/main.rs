use entrypoint::handle_result;

mod entrypoint;
mod run;
mod taskfile;
mod tests;
mod utils;

fn main() {
    let entrypoint = entrypoint::EntryPoint::new();
    handle_result(entrypoint.run());
}
