use hex_literal::hex;

// the generator ROM from:
// https://github.com/Chik-Network/chik-blockchain/blob/main/chik/wallet/puzzles/rom_bootstrap_generator.clsp.hex
pub const GENERATOR_ROM: [u8; 737] = hex!(
    "
    ff02ffff01ff02ff0cffff04ff02ffff04ffff02ff05ffff04ff08ffff04ff13
    ff80808080ff80808080ffff04ffff01ffffff02ffff01ff05ffff02ff3effff
    04ff02ffff04ff05ff8080808080ffff04ffff01ffffff81ff7fff81df81bfff
    ffff02ffff03ffff09ff0bffff01818080ffff01ff04ff80ffff04ff05ff8080
    80ffff01ff02ffff03ffff0aff0bff1880ffff01ff02ff1affff04ff02ffff04
    ffff02ffff03ffff0aff0bff1c80ffff01ff02ffff03ffff0aff0bff1480ffff
    01ff0880ffff01ff04ffff0effff18ffff011fff0b80ffff0cff05ff80ffff01
    018080ffff04ffff0cff05ffff010180ff80808080ff0180ffff01ff04ffff18
    ffff013fff0b80ffff04ff05ff80808080ff0180ff80808080ffff01ff04ff0b
    ffff04ff05ff80808080ff018080ff0180ff04ffff0cff15ff80ff0980ffff04
    ffff0cff15ff0980ff808080ffff04ffff04ff05ff1380ffff04ff2bff808080
    ffff02ff16ffff04ff02ffff04ff09ffff04ffff02ff3effff04ff02ffff04ff
    15ff80808080ff8080808080ff02ffff03ffff09ffff0cff05ff80ffff010180
    ff1080ffff01ff02ff2effff04ff02ffff04ffff02ff3effff04ff02ffff04ff
    ff0cff05ffff010180ff80808080ff80808080ffff01ff02ff12ffff04ff02ff
    ff04ffff0cff05ffff010180ffff04ffff0cff05ff80ffff010180ff80808080
    8080ff0180ff018080ff04ffff02ff16ffff04ff02ffff04ff09ff80808080ff
    0d80ffff04ff09ffff04ffff02ff1effff04ff02ffff04ff15ff80808080ffff
    04ff2dffff04ffff02ff15ff5d80ff7d80808080ffff02ffff03ff05ffff01ff
    04ffff02ff0affff04ff02ffff04ff09ff80808080ffff02ff16ffff04ff02ff
    ff04ff0dff8080808080ff8080ff0180ff02ffff03ffff07ff0580ffff01ff0b
    ffff0102ffff02ff1effff04ff02ffff04ff09ff80808080ffff02ff1effff04
    ff02ffff04ff0dff8080808080ffff01ff0bffff0101ff058080ff0180ff0180
    80"
);

// the KLVM deserializer from:
// https://github.com/Chik-Network/chik-blockchain/blob/main/chik/wallet/puzzles/chiklisp_deserialisation.clsp.hex
pub const KLVM_DESERIALIZER: [u8; 471] = hex!(
"
ff02ffff01ff05ffff02ff3effff04ff02ffff04ff05ff8080808080ffff04ffff01ffffff81ff7fff81df81bfffffff02ffff03ffff09ff0bffff01818080ffff01ff04ff80ffff04ff05ff808080ffff01ff02ffff03ffff0aff0bff1880ffff01ff02ff1affff04ff02ffff04ffff02ffff03ffff0aff0bff1c80ffff01ff02ffff03ffff0aff0bff1480ffff01ff0880ffff01ff04ffff0effff18ffff011fff0b80ffff0cff05ff80ffff01018080ffff04ffff0cff05ffff010180ff80808080ff0180ffff01ff04ffff18ffff013fff0b80ffff04ff05ff80808080ff0180ff80808080ffff01ff04ff0bffff04ff05ff80808080ff018080ff0180ff04ffff0cff15ff80ff0980ffff04ffff0cff15ff0980ff808080ffff04ffff04ff05ff1380ffff04ff2bff808080ffff02ff16ffff04ff02ffff04ff09ffff04ffff02ff3effff04ff02ffff04ff15ff80808080ff8080808080ff02ffff03ffff09ffff0cff05ff80ffff010180ff1080ffff01ff02ff2effff04ff02ffff04ffff02ff3effff04ff02ffff04ffff0cff05ffff010180ff80808080ff80808080ffff01ff02ff12ffff04ff02ffff04ffff0cff05ffff010180ffff04ffff0cff05ff80ffff010180ff808080808080ff0180ff018080"
);

// constant from the main chik blockchain:
// https://github.com/Chik-Network/chik-blockchain/blob/main/chik/consensus/default_constants.py
pub const COST_PER_BYTE: u64 = 12000;
