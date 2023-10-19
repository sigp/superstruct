use serde::{Deserialize, Serialize};
use superstruct::superstruct;

#[superstruct(
    variants(V1, V2, V3),
    variant_attributes(derive(Deserialize, Serialize))
)]
#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub struct Customer {
    pub name: String,
    #[superstruct(only(V1), partial_getter(rename = "age_v1"))]
    pub age: String,
    #[superstruct(only(V2), partial_getter(rename = "age_v2"))]
    pub age: u64,
    #[superstruct(only(V3))]
    pub dob: u64,
    #[superstruct(only(V2, V3))]
    pub favourite_colour: String,
}

fn main() {
    let customer = Customer::V3(CustomerV3 {
        name: "Michael".into(),
        dob: 0,
        favourite_colour: "purple".into(),
    });
    assert_eq!(customer.name(), "Michael");
    assert_eq!(customer.dob(), Ok(&0));
    assert_eq!(customer.favourite_colour().unwrap(), "purple");
}
