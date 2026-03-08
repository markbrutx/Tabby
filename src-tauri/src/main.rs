use clap::Parser;

fn main() {
    tabby_app_lib::run(tabby_app_lib::CliArgs::parse());
}
