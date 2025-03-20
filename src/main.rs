use std::process;

fn main() {
    json_parser::run().unwrap_or_else(|err| {
        eprintln!("{err}");
        process::exit(1);
    })
}
