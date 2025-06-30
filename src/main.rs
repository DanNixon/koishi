mod cli;
mod secret_store;
mod utils;

fn main() -> miette::Result<()> {
    cli::main()
}
