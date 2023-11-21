use chik_streamable_macro::Streamable;

use crate::streamable_struct;

streamable_struct!(FeeRate {
    // Represents Fee Rate in mojos divided by KLVM Cost.
    // Performs XCK/mojo conversion.
    // Similar to 'Fee per cost'.
    mojos_per_klvm_cost: u64,
});

streamable_struct! (FeeEstimate {
    error: Option<String>,
    time_target: u64,            // unix time stamp in seconds
    estimated_fee_rate: FeeRate, // Mojos per klvm cost
});

streamable_struct! (FeeEstimateGroup {
    error: Option<String>,
    estimates: Vec<FeeEstimate>,
});
