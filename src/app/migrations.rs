use ron::{Value, value::{Map, Number, F32}};

// This is currently unused due to issues with RON deserializing into Value type without losing version information
#[allow(dead_code)]
fn migrate_v2(editor: &mut Value) -> anyhow::Result<()> {
    // new entry node_sized needs to be populated with node_ids in the graph.
    if let Value::Map(state) = editor {
        let nodes = &state[&Value::String("node_order".to_string())];
        if let Value::Map(nodes) = nodes {
            let mut node_sizes = Value::Map(Map::new());
            let node_sizes_map = match &mut node_sizes {
                Value::Map(map) => map,
                _ => unreachable!("Can't be not a map"),
            };
            for (version, _) in nodes.iter() {
                let mut sizes_map = Map::new();
                sizes_map.insert(
                    Value::String("x".into()),
                    Value::Number(Number::F32(F32::new(200.0)))
                );
                sizes_map.insert(
                    Value::String("y".into()),
                    Value::Number(Number::F32(F32::new(200.0)))
                );
                let opt = Value::Option(Some(Box::from(Value::Map(sizes_map))));
                node_sizes_map.insert(Value::String("value".into()), opt);
                node_sizes_map.insert(Value::String("version".into()), version.clone());
            }
            state.insert(ron::Value::String("node_sizes".to_string()), node_sizes);
        }
    }
    Ok(())
}