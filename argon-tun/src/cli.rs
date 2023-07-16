use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(long)]
    pub mtu: Option<u16>,

    #[arg(long)]
    pub channel_size: usize,
}
