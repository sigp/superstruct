// TODO: make this into a test
use serde::{Deserialize, Serialize};
use superstruct::superstruct;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
enum ForkName {
    Bellatrix,
    Capella,
    Deneb,
    Electra,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
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
    variant_type(name = "ForkName", getter = "fork_name"),
    feature_type(
        name = "FeatureName",
        list = "feature_names",
        check = "check_feature_enabled"
    )
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

#[superstruct(
    feature(Merge),
    variants_and_features_from = "FORK_ORDER",
    feature_dependencies = "FEATURE_DEPENDENCIES",
    variant_type(name = "ForkName", getter = "fork_name"),
    feature_type(
        name = "FeatureName",
        list = "feature_names",
        check = "check_feature_enabled"
    )
)]
struct Payload {
    transactions: Vec<u64>,
}

fn main() {
    let block = Block::Electra(BlockElectra {
        historical_updates: "hey".into(),
        historical_summaries: "thing".into(),
        withdrawals: vec![1, 2, 3],
        deposits: vec![0, 0, 0, 0, 0, 0],
    });

    assert_eq!(block.fork_name(), ForkName::Electra);
    assert_eq!(
        block.feature_names(),
        vec![
            FeatureName::Merge,
            FeatureName::Withdrawals,
            FeatureName::EIP6110,
            FeatureName::Verge
        ]
    );
    assert!(block.check_feature_enabled(FeatureName::EIP6110));
}
