use superstruct::superstruct;

enum ForkName {
    Bellatrix,
    Capella,
    Deneb,
    Electra,
}

enum FeatureName {
    Merge,
    Withdrawals,
    Blobs,
    EIP6110,
    Verge,
}

#[superstruct(FORK_ORDER)]
const FORK_ORDER: Vec<(ForkName, Vec<FeatureName>)> = vec![
    (ForkName::Bellatrix, vec![FeatureName::Merge]),
    (ForkName::Capella, vec![FeatureName::Withdrawals]),
    (
        ForkName::Electra,
        vec![FeatureName::EIP6110, FeatureName::Verge],
    ),
];

const FEATURE_DEPENDENCIES: Vec<(FeatureName, Vec<FeatureName>)> = vec![
    (FeatureName::Withdrawals, vec![FeatureName::Merge]),
    (FeatureName::Blobs, vec![FeatureName::Withdrawals]),
    (FeatureName::EIP6110, vec![FeatureName::Merge]),
    (FeatureName::Verge, vec![FeatureName::Merge]),
];

#[superstruct(
    variants_and_features_from(FORK_NAME),
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
