use clap::Args;

#[derive(Args, Debug)]
pub struct TransactionFlags {
    /// Maximum amount of gas units to be used to send this transaction
    ///
    /// The maximum amount of gas units willing to pay for the transaction. This is the (max gas in coins / gas unit price).
    ///
    /// For examples if I wanted to pay a maximum of 100 coins, I may have the max gas set to 100 if the gas unit price is 1.  
    /// If I want it to have a gas unit price of 2, the max gas would need to be 50 to still only have a maximum price of 100 coins.
    ///
    /// Without a value, it will determine the price based on simulating the current transaction
    #[clap(long = "max-gas", default_value = "20000", value_parser)]
    pub max_gas: u32,
}
