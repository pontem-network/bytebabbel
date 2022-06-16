use clap::Parser;
use serde::Deserialize;


#[derive(Parser, Debug, Clone, PartialEq)]
// #[clap(author, version, about, long_about = None)]
pub struct Cfg {
	#[clap(long)]
	address: String,
}

#[derive(Parser, Debug, Clone, PartialEq)]
// #[clap(author, version, about, long_about = None)]
pub struct CfgOverride {
	#[clap(long)]
	address: Option<String>,
}



#[derive(Debug, Deserialize)]
pub struct SolAbi {

}
