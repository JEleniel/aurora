use clap::Parser;

#[derive(Debug, Parser)]
pub struct BumpArgs {
	#[arg(short, long = "card", value_name = "CARD_ID", help = "Card id to bump")]
	pub card_id: String,
	#[arg(
		short,
		long = "editor",
		value_name = "NAME",
		help = "Override the audit history editor"
	)]
	pub editor: Option<String>,
}
