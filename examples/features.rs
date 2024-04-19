use serde::{Deserialize, Serialize};
use superstruct::superstruct;

#[derive(Serialize, Deserialize)]
enum ForkName {
    Bellatrix,
    Capella,
    Deneb,
    Electra,
}

#[derive(Serialize, Deserialize)]
enum FeatureName {
    Merge,
    Withdrawals,
    Blobs,
    EIP6110,
    Verge,
    EIP7549,
}

#[superstruct(variants_and_features_decl = "FORK_ORDER")]
const FORK_ORDER: &[(ForkName, &[FeatureName])] = &[
    (ForkName::Bellatrix, &[FeatureName::Merge]),
    (ForkName::Capella, &[FeatureName::Withdrawals]),
    (
        ForkName::Electra,
        &[FeatureName::EIP6110, FeatureName::Verge],
    ),
];

#[superstruct(feature_dependencies_decl = "FEATURE_DEPENDENCIES")]
const FEATURE_DEPENDENCIES: &[(FeatureName, &[FeatureName])] = &[
    (FeatureName::Withdrawals, &[FeatureName::Merge]),
    (FeatureName::Blobs, &[FeatureName::Withdrawals]),
    (FeatureName::EIP6110, &[FeatureName::Merge]),
    (FeatureName::Verge, &[FeatureName::Merge]),
];

#[superstruct(
    variants_and_features_from = "FORK_ORDER",
    feature_dependencies = "FEATURE_DEPENDENCIES",
    variant_type(ForkName),
    feature_type(FeatureName)
)]
struct Block {
    historical_updates: String,
    #[superstruct(feature(Withdrawals))]
    historical_summaries: String,
    #[superstruct(feature(Withdrawals))] // in the Withdrawals fork, and all subsequent
    withdrawals: Vec<u64>,
    #[superstruct(feature(Blobs))] // if Blobs is not enabled, this is completely disabled
    blobs: Vec<u64>,
    #[superstruct(feature(EIP6110))]
    deposits: Vec<u64>,
}

// Should generate this:
/*
impl Block {
    fn fork_name(&self) -> ForkName;

    fn feature_names(&self) -> &'static [FeatureName];

    fn is_feature_enabled(&self, feature: FeatureName) -> bool {
        match self {
            Self::Capella => false,
            Self::Electra => true,
        }
    }
}
*/

fn main() {}
