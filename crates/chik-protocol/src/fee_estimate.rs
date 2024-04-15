use chik_streamable_macro::streamable;

#[streamable]
pub struct FeeRate {
    // Represents Fee Rate in mojos divided by KLVM Cost.
    // Performs XCK/mojo conversion.
    // Similar to 'Fee per cost'.
    mojos_per_klvm_cost: u64,
}

#[streamable]
pub struct FeeEstimate {
    error: Option<String>,
    time_target: u64,            // unix time stamp in seconds
    estimated_fee_rate: FeeRate, // Mojos per klvm cost
}

#[streamable]
pub struct FeeEstimateGroup {
    error: Option<String>,
    estimates: Vec<FeeEstimate>,
}
