use dap_reactor::prelude::{
    Source, SourceReference, Variable, VariablePresentationHint,
    VariablePresentationHintAttribute, VariablePresentationHintKind,
    VariablePresentationHintVisibility,
};

use crate::{Constraint, Scalar, Witness};

impl From<&Constraint<'_>> for Source {
    fn from(constraint: &Constraint) -> Self {
        let path = constraint.name().to_string();

        Source {
            name: Some(path.clone()),
            source_reference: Some(SourceReference::Path(path)),
            presentation_hint: None,
            origin: None,
            sources: vec![],
            adapter_data: None,
            checksums: vec![],
        }
    }
}

pub fn scalar_to_string(scalar: &Scalar) -> String {
    format!("0x{}", hex::encode(scalar.as_ref()))
}

pub fn scalar_to_var<N>(name: N, scalar: &Scalar) -> Variable
where
    N: Into<String>,
{
    Variable {
        name: name.into(),
        value: scalar_to_string(scalar),
        r#type: Some("scalar".into()),
        presentation_hint: Some(VariablePresentationHint {
            kind: Some(VariablePresentationHintKind::Data),
            attributes: vec![VariablePresentationHintAttribute::Constant],
            visibility: Some(VariablePresentationHintVisibility::Protected),
            lazy: false,
        }),
        evaluate_name: None,
        variables_reference: 0,
        named_variables: None,
        indexed_variables: None,
        memory_reference: None,
    }
}

pub fn idx_to_var<N>(name: N, idx: usize) -> Variable
where
    N: Into<String>,
{
    Variable {
        name: name.into(),
        value: idx.to_string(),
        r#type: Some("usize".into()),
        presentation_hint: Some(VariablePresentationHint {
            kind: Some(VariablePresentationHintKind::Data),
            attributes: vec![VariablePresentationHintAttribute::Constant],
            visibility: Some(VariablePresentationHintVisibility::Protected),
            lazy: false,
        }),
        evaluate_name: None,
        variables_reference: 0,
        named_variables: None,
        indexed_variables: None,
        memory_reference: None,
    }
}

pub fn witness_to_var<N>(name: N, witness: Witness) -> Variable
where
    N: Into<String>,
{
    Variable {
        name: name.into(),
        value: serde_json::json!({
            "id": witness.id(),
            "value": scalar_to_string(witness.value()),
            "constraint": witness
                .constraint(),
                "source": witness.name(),
                "line": witness.line(),
        })
        .to_string(),
        r#type: Some("scalar".into()),
        presentation_hint: Some(VariablePresentationHint {
            kind: Some(VariablePresentationHintKind::Data),
            attributes: vec![VariablePresentationHintAttribute::Constant],
            visibility: Some(VariablePresentationHintVisibility::Protected),
            lazy: false,
        }),
        evaluate_name: None,
        variables_reference: 0,
        named_variables: None,
        indexed_variables: None,
        memory_reference: None,
    }
}

pub fn bool_to_var<N>(name: N, b: bool) -> Variable
where
    N: Into<String>,
{
    Variable {
        name: name.into(),
        value: b.to_string(),
        r#type: Some("bool".into()),
        presentation_hint: Some(VariablePresentationHint {
            kind: Some(VariablePresentationHintKind::Data),
            attributes: vec![VariablePresentationHintAttribute::ReadOnly],
            visibility: Some(VariablePresentationHintVisibility::Protected),
            lazy: false,
        }),
        evaluate_name: None,
        variables_reference: 0,
        named_variables: None,
        indexed_variables: None,
        memory_reference: None,
    }
}
