use super::*;
use crate::bindings::{BindingEnum, EnumVariant};
use crate::typing::{EnumBindingIndex, EnumBindingValueIndex};

#[test]
fn generic_enum_operand_compiles_with_selected_category_and_variant() {
    let mut graph = PulseGraph::default();
    let mut state = PulseGraphState::default();
    state.bindings.enums.push(BindingEnum {
        id: EnumBindingIndex(7),
        name: "Mode".into(),
        name_ui: "Mode".into(),
        variants: vec![
            EnumVariant {
                name: "Disabled".into(),
                name_ui: "Disabled".into(),
            },
            EnumVariant {
                name: "Enabled".into(),
                name_ui: "Enabled".into(),
            },
        ],
    });
    let node = graph.add_node("Compare".into(), Default::default(), |_, _| {});
    let input = graph.add_input_param(
        node,
        "A".into(),
        PulseDataType::SchemaEnum,
        PulseGraphValueType::SchemaEnumChoice {
            enum_type: EnumBindingIndex(7),
            enum_variant: EnumBindingValueIndex(1),
        },
        InputParamKind::ConnectionOrConstant,
        true,
    );
    let mut compiled = PulseGraphDef::default();
    compiled.chunks.push(PulseChunk::default());
    let register = get_input_register_or_create_constant_from_id(
        &graph,
        &mut compiled,
        &state,
        0,
        input,
        PulseValueType::PVAL_SCHEMA_ENUM_INDEXED(None, None),
        false,
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        compiled.chunks[0].get_register_type(register),
        Some("PVAL_SCHEMA_ENUM:Mode")
    );
    assert!(
        matches!(&compiled.constants[0], PulseConstant::SchemaEnumValue(category, variant)
        if category == "Mode" && variant == "Enabled")
    );
}
