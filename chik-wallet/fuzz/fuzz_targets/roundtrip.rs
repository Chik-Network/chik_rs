#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    #[cfg(fuzzing)]
    {
        use std::fmt;

        use chik_wallet::{nft::NftMetadata, Proof};
        use klvm_traits::{FromKlvm, ToKlvm};
        use klvmr::{allocator::NodePtr, Allocator};
        use libfuzzer_sys::arbitrary::{Arbitrary, Unstructured};

        let mut u = Unstructured::new(data);
        roundtrip::<NftMetadata>(&mut u);
        roundtrip::<Proof>(&mut u);

        fn roundtrip<'a, T>(u: &mut Unstructured<'a>)
        where
            T: Arbitrary<'a> + ToKlvm<NodePtr> + FromKlvm<NodePtr> + PartialEq + fmt::Debug,
        {
            let obj = T::arbitrary(u).unwrap();
            let mut a = Allocator::new();
            let ptr = obj.to_klvm(&mut a).unwrap();
            let obj2 = T::from_klvm(&a, ptr).unwrap();
            assert_eq!(obj, obj2);
        }
    }

    #[cfg(not(fuzzing))]
    {
        let _ = data;
    }
});
