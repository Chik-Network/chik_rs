use crate::coin_spend::CoinSpend;
use crate::streamable_struct;
use chik_bls::G2Element;
use chik_streamable_macro::Streamable;

streamable_struct! (SpendBundle {
    coin_spends: Vec<CoinSpend>,
    aggregated_signature: G2Element,
});
