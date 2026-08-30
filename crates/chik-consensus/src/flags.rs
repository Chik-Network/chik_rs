use bitflags::bitflags;
use klvmr::MEMPOOL_MODE as KLVM_MEMPOOL_MODE;

#[cfg(feature = "py-bindings")]
use pyo3::{Bound, FromPyObject, IntoPyObject, PyAny, PyErr, PyResult, Python, types::PyInt};

bitflags! {
    /// Full flag set for KLVM execution and consensus (condition parsing, validation, generator mode).
    /// Combines flags from klvmr (lower bytes) and consensus (upper bytes).
    /// The end goal should be to make these flags independent, but we still
    /// have at least one quirk in chik-protocol's Program::run_rust() where it
    /// would be ideal to take Consensusflags, but it can't depend on
    /// chik-consensus, so it has to take KlvmFlags instead. those aren't exposed
    /// to python, so it relies on these flags matching.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ConsensusFlags: u32 {
        // Flags from klvmr (chik_dialect)
        // we still rely on these bits matching exactly the flags in klvm_rs
        // via the python binding, which "launders" the type of the flags
        const CANONICAL_INTS = 0x0001;
        const NO_UNKNOWN_OPS = 0x0002;
        const LIMIT_HEAP = 0x0004;
        const RELAXED_BLS = 0x0008;
        const LIMITS = 0x0010;
        const ENABLE_GC = 0x0020;
        const ENABLE_KECCAK_OPS_OUTSIDE_GUARD = 0x0100;
        const DISABLE_OP = 0x0200;
        const ENABLE_SHA256_TREE = 0x0400;
        const ENABLE_SECP_OPS = 0x0800;
        const MALACHITE = 0x1000;

        // Consensus flags
        /// Skip validating AGG_SIG / condition signatures.
        const DONT_VALIDATE_SIGNATURE = 0x1_0000;

        /// Unknown condition codes are disallowed (mempool-mode).
        const NO_UNKNOWN_CONDS = 0x2_0000;

        /// Compute condition fingerprints for spends eligible for dedup.
        const COMPUTE_FINGERPRINT = 0x4_0000;

        /// Conditions require the exact supported argument count (mempool-mode).
        const STRICT_ARGS_COUNT = 0x8_0000;

        /// Add flat cost to conditions (active after hard fork 2).
        const COST_CONDITIONS = 0x80_0000;

        /// Simpler generator rules (hard fork behavior).
        const SIMPLE_GENERATOR = 0x100_0000;

        /// Limit the number of spends per block.
        const LIMIT_SPENDS = 0x200_0000;

        /// After the generator-identity hard fork, generators must be validated from
        /// the INTERNED (canonical) tree so atom/pair limits and cost apply to the same
        /// structure independent of serialization.
        const INTERNED_GENERATOR = 0x0800_0000;
    }
}

impl ConsensusFlags {
    /// Convert klvmr's KlvmFlags to the corresponding ConsensusFlags (shared flags only).
    /// For each klvmr flag we check whether it is set (using contains()), then set our corresponding flag.
    #[must_use]
    const fn from_klvm_flags(klvm: klvmr::chik_dialect::KlvmFlags) -> Self {
        use klvmr::chik_dialect::KlvmFlags;
        let mut out = ConsensusFlags::empty();
        if klvm.contains(KlvmFlags::CANONICAL_INTS) {
            out = out.union(ConsensusFlags::CANONICAL_INTS);
        }
        if klvm.contains(KlvmFlags::NO_UNKNOWN_OPS) {
            out = out.union(ConsensusFlags::NO_UNKNOWN_OPS);
        }
        if klvm.contains(KlvmFlags::LIMIT_HEAP) {
            out = out.union(ConsensusFlags::LIMIT_HEAP);
        }
        if klvm.contains(KlvmFlags::RELAXED_BLS) {
            out = out.union(ConsensusFlags::RELAXED_BLS);
        }
        if klvm.contains(KlvmFlags::LIMITS) {
            out = out.union(ConsensusFlags::LIMITS);
        }
        if klvm.contains(KlvmFlags::ENABLE_GC) {
            out = out.union(ConsensusFlags::ENABLE_GC);
        }
        if klvm.contains(KlvmFlags::ENABLE_KECCAK_OPS_OUTSIDE_GUARD) {
            out = out.union(ConsensusFlags::ENABLE_KECCAK_OPS_OUTSIDE_GUARD);
        }
        if klvm.contains(KlvmFlags::DISABLE_OP) {
            out = out.union(ConsensusFlags::DISABLE_OP);
        }
        if klvm.contains(KlvmFlags::ENABLE_SHA256_TREE) {
            out = out.union(ConsensusFlags::ENABLE_SHA256_TREE);
        }
        if klvm.contains(KlvmFlags::ENABLE_SECP_OPS) {
            out = out.union(ConsensusFlags::ENABLE_SECP_OPS);
        }
        if klvm.contains(KlvmFlags::MALACHITE) {
            out = out.union(ConsensusFlags::MALACHITE);
        }
        out
    }

    /// Convert to klvmr's KlvmFlags by mapping each shared flag to its KlvmFlags counterpart.
    /// Does not rely on underlying bits being the same; consensus-only flags are ignored.
    pub fn to_klvm_flags(self) -> klvmr::chik_dialect::KlvmFlags {
        use klvmr::chik_dialect::KlvmFlags;
        let mut out = KlvmFlags::empty();
        if self.contains(ConsensusFlags::CANONICAL_INTS) {
            out.insert(KlvmFlags::CANONICAL_INTS);
        }
        if self.contains(ConsensusFlags::NO_UNKNOWN_OPS) {
            out.insert(KlvmFlags::NO_UNKNOWN_OPS);
        }
        if self.contains(ConsensusFlags::LIMIT_HEAP) {
            out.insert(KlvmFlags::LIMIT_HEAP);
        }
        if self.contains(ConsensusFlags::RELAXED_BLS) {
            out.insert(KlvmFlags::RELAXED_BLS);
        }
        if self.contains(ConsensusFlags::LIMITS) {
            out.insert(KlvmFlags::LIMITS);
        }
        if self.contains(ConsensusFlags::ENABLE_GC) {
            out.insert(KlvmFlags::ENABLE_GC);
        }
        if self.contains(ConsensusFlags::ENABLE_KECCAK_OPS_OUTSIDE_GUARD) {
            out.insert(KlvmFlags::ENABLE_KECCAK_OPS_OUTSIDE_GUARD);
        }
        if self.contains(ConsensusFlags::DISABLE_OP) {
            out.insert(KlvmFlags::DISABLE_OP);
        }
        if self.contains(ConsensusFlags::ENABLE_SHA256_TREE) {
            out.insert(KlvmFlags::ENABLE_SHA256_TREE);
        }
        if self.contains(ConsensusFlags::ENABLE_SECP_OPS) {
            out.insert(KlvmFlags::ENABLE_SECP_OPS);
        }
        if self.contains(ConsensusFlags::MALACHITE) {
            out.insert(KlvmFlags::MALACHITE);
        }
        out
    }
}

/// Mempool-mode: klvmr MEMPOOL_MODE plus consensus stricter checking.
pub const MEMPOOL_MODE: ConsensusFlags = ConsensusFlags::from_klvm_flags(KLVM_MEMPOOL_MODE)
    .union(ConsensusFlags::NO_UNKNOWN_CONDS)
    .union(ConsensusFlags::STRICT_ARGS_COUNT)
    .union(ConsensusFlags::LIMIT_SPENDS);

impl Default for ConsensusFlags {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(feature = "py-bindings")]
impl<'py> FromPyObject<'py, 'py> for ConsensusFlags {
    type Error = PyErr;

    fn extract(obj: pyo3::Borrowed<'py, 'py, PyAny>) -> PyResult<Self> {
        let b: u32 = obj.extract()?;
        Ok(ConsensusFlags::from_bits_truncate(b))
    }
}

#[cfg(feature = "py-bindings")]
impl<'py> IntoPyObject<'py> for ConsensusFlags {
    type Target = PyInt;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyInt::new(py, self.bits()))
    }
}

#[cfg(test)]
mod tests {
    use super::ConsensusFlags;
    use bitflags::Flags;
    use klvmr::chik_dialect::KlvmFlags;

    /// No two flags may share any bit
    #[test]
    fn no_overlapping_bits() {
        let all = ConsensusFlags::FLAGS;
        for (i, a) in all.iter().enumerate() {
            for b in &all[i + 1..] {
                let a_bits = a.value().bits();
                let b_bits = b.value().bits();
                assert_eq!(
                    a_bits & b_bits,
                    0,
                    "overlapping bits between {:?} ({a_bits:x}) and {:?} ({b_bits:x})",
                    a.value(),
                    b.value(),
                );
            }
        }
    }

    /// Every KlvmFlags flag must exist in ConsensusFlags and have the exact same bits.
    /// We rely on this for the python binding (Program::run_rust) which launders flags as u32.
    #[test]
    fn klvm_flags_bits_match_consensus_flags() {
        let klvm_flags = KlvmFlags::FLAGS;
        for flag in klvm_flags {
            assert!(flag.is_named());
            let name = flag.name();
            let klvm_bits = flag.value().bits();
            let Some(consensus) = ConsensusFlags::from_name(name) else {
                panic!(
                    "KlvmFlags flag {name} has no corresponding ConsensusFlags; \
                     every KlvmFlags flag must exist in ConsensusFlags"
                )
            };
            assert_eq!(
                klvm_bits,
                consensus.bits(),
                "KlvmFlags and ConsensusFlags must have the same bits for flag {:?} (name = {name}); \
                 we rely on exact bit compatibility",
                flag.value(),
            );
        }
    }

    /// Every shared flag round-trips through from_klvm_flags / to_klvm_flags,
    /// and consensus-only flags never leak into KlvmFlags.
    #[test]
    fn shared_flags_round_trip_through_conversion() {
        for flag in KlvmFlags::FLAGS {
            assert!(flag.is_named());
            let klvm = *flag.value();
            let name = flag.name();

            let consensus = ConsensusFlags::from_klvm_flags(klvm);
            let expected = ConsensusFlags::from_name(name).unwrap();
            assert_eq!(
                consensus, expected,
                "from_klvm_flags did not convert KlvmFlags::{name} correctly"
            );

            let back = expected.to_klvm_flags();
            assert_eq!(
                back, klvm,
                "to_klvm_flags did not convert ConsensusFlags::{name} back to KlvmFlags::{name}"
            );
        }

        // ConsensusFlags is a strict superset: consensus-only flags exist
        // and must not leak into KlvmFlags via to_klvm_flags.
        let consensus_only =
            ConsensusFlags::all().difference(ConsensusFlags::from_klvm_flags(KlvmFlags::all()));
        assert!(
            !consensus_only.is_empty(),
            "ConsensusFlags should be a strict superset of KlvmFlags"
        );
        assert!(
            consensus_only.to_klvm_flags().is_empty(),
            "consensus-only flags must not appear in to_klvm_flags output"
        );
    }
}
