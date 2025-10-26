use ndate::parse_datetime;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The string to parse. Supports multiple formats:
    ///
    /// - Absolute date: "2024-10-25", "24-10-25"
    /// - Relative date: "+1"/"1" (1 day later), "-3" (3 days ago)
    /// - Partial date: "10-25" (month-day, automatically infer the year)
    #[arg(verbatim_doc_comment, allow_hyphen_values = true)]
    date_input: String,

    /// Treat the input as a lunar date for processing
    #[arg(short = 'l', long = "lunar_date")]
    lunar_date: bool,

    /// Custom date format string
    #[arg(short = 'f', long = "format")]
    format: Option<String>,

    /// For partial dates (MM-DD), disable automatically using the next year
    /// Default behavior: if the date has passed in the current year, use the next year
    #[arg(long = "no-next", verbatim_doc_comment)]
    no_next: bool,
}

fn main() {
    let cli = Cli::parse();
    let output = parse_datetime(cli.date_input, cli.lunar_date, !cli.no_next, &cli.format);
    println!("{}", output);
}
