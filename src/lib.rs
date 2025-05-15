use clap::ValueEnum;

#[derive(Clone, Debug, Default, ValueEnum)]
pub enum DateFormat {
    #[default]
    Absolute,
    Relative,
}
