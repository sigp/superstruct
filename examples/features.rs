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
}

#[superstruct(variants_and_features_decl = "fork_order")]
const FORK_ORDER: &[(ForkName, &[FeatureName])] = &[
    (ForkName::Bellatrix, &[FeatureName::Merge]),
    (ForkName::Capella, &[FeatureName::Withdrawals]),
    (
        ForkName::Electra,
        &[FeatureName::EIP6110, FeatureName::Verge],
    ),
];

#[superstruct(feature_dependencies_decl = "feature_dependencies")]
const FEATURE_DEPENDENCIES: &[(FeatureName, &[FeatureName])] = &[
    (FeatureName::Withdrawals, &[FeatureName::Merge]),
    (FeatureName::Blobs, &[FeatureName::Withdrawals]),
    (FeatureName::EIP6110, &[FeatureName::Merge]),
    (FeatureName::Verge, &[FeatureName::Merge]),
];

#[superstruct(
    variants_and_features_from(FORK_ORDER),
    feature_dependencies(FEATURE_DEPENDENCIES),
    variant_type(ForkName),
    feature_type(FeatureName)
)]
struct Block {
    #[until(Withdrawals)] // until the fork that has Withdrawals feature
    historical_updates: String,
    #[from(Withdrawals)]
    historical_summaries: String,
    #[from(Withdrawals)] // in the Withdrawals fork, and all subsequent
    withdrawals: Vec<u64>,
    #[from(Blobs)] // if Blobs is not enabled, this is completely disabled
    blobs: Vec<u64>,
    #[from(EIP6110)]
    deposits: Vec<u64>,
}
// TODO: try some variants as well

fn main() {}
