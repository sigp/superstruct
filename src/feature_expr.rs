use darling::{Error, FromMeta};
use syn::{Ident, Meta, NestedMeta};

/// A cfg-like expression in terms of features, which can be evaluated for each fork at each field
/// to determine whether that field is turned on.
#[derive(Debug)]
pub enum FeatureExpr {
    And(Box<FeatureExpr>, Box<FeatureExpr>),
    Or(Box<FeatureExpr>, Box<FeatureExpr>),
    Not(Box<FeatureExpr>),
    Literal(Ident),
}

fn parse(meta: NestedMeta) -> Result<FeatureExpr, Error> {
    match meta {
        // TODO: assert 1 segment
        NestedMeta::Meta(Meta::Path(path)) => Ok(FeatureExpr::Literal(
            path.segments.last().unwrap().ident.clone(),
        )),
        NestedMeta::Meta(Meta::List(meta_list)) => {
            let segments = &meta_list.path.segments;
            assert_eq!(segments.len(), 1);
            let operator = &segments.last().unwrap().ident;
            match operator.to_string().as_str() {
                "and" => {
                    let mut nested = meta_list.nested;
                    assert_eq!(nested.len(), 2, "`and` should have exactly 2 operands");
                    let right_meta = nested.pop().unwrap().into_value();
                    let left_meta = nested.pop().unwrap().into_value();
                    Ok(FeatureExpr::And(
                        Box::new(parse(left_meta)?),
                        Box::new(parse(right_meta)?),
                    ))
                }
                "or" => {
                    let mut nested = meta_list.nested;
                    assert_eq!(nested.len(), 2, "`or` should have exactly 2 operands");
                    let right_meta = nested.pop().unwrap().into_value();
                    let left_meta = nested.pop().unwrap().into_value();
                    Ok(FeatureExpr::Or(
                        Box::new(parse(left_meta)?),
                        Box::new(parse(right_meta)?),
                    ))
                }
                "not" => {
                    let mut nested = meta_list.nested;
                    assert_eq!(nested.len(), 1, "`not` should have exactly 1 operand");
                    let inner_meta = nested.pop().unwrap().into_value();
                    Ok(FeatureExpr::Not(Box::new(parse(inner_meta)?)))
                }
                op => panic!("unsupported operator: {op}"),
            }
        }
        _ => panic!("unexpected feature expr: {meta:?}"),
    }
}

impl FromMeta for FeatureExpr {
    fn from_list(items: &[NestedMeta]) -> Result<Self, Error> {
        assert_eq!(items.len(), 1, "feature expr should have 1 part");
        let expr_meta = items.first().cloned().unwrap();
        parse(expr_meta)
    }
}

impl FeatureExpr {
    pub fn eval(&self, features: &[Ident]) -> bool {
        match self {
            Self::Literal(feature_name) => features.contains(&feature_name),
            Self::And(left, right) => left.eval(features) && right.eval(features),
            Self::Or(left, right) => left.eval(features) || right.eval(features),
            Self::Not(inner) => !inner.eval(features),
        }
    }
}
