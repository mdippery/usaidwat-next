use clap::Parser;
use usaidwat::Config;

fn main() {
    let config = Config::parse();
    usaidwat::run(config)
}
