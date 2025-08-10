use egui_node_graph2::*;
use anyhow::anyhow;
use crate::app::types::*;

// returns list of pairs of nodes and inputs connected to a given output.
// TODO could be optimized to return a group of node and it's inputs
pub fn get_nodes_and_inputs_connected_from_output(
    graph: &PulseGraph,
    out_action_id: &OutputId,
) -> anyhow::Result<Vec<(NodeId, InputId)>> {
    let mut node_input_pairs = vec![];
    for connection in graph.iter_connections() {
        if connection.1 == *out_action_id {
            let node_of_input = graph
                .inputs
                .get(connection.0)
                .ok_or(anyhow!("Can't find input value {:?}", connection.0))?
                .node();
            let input_id = connection.0;
            node_input_pairs.push((node_of_input, input_id));
        }
    }
    Ok(node_input_pairs)
}

// return list of pairs of the connected node and corresponding name
pub fn get_nodes_connected_to_output<'a>(
    origin_node: &'a Node<PulseNodeData>,
    graph: &'a PulseGraph,
    name: &str,
) -> anyhow::Result<Vec<(&'a Node<PulseNodeData>, &'a str)>> {
    let mut res = vec![];
    let out_action_id = origin_node.get_output(name)?;
    let connected_nodes_inputs = get_nodes_and_inputs_connected_from_output(graph, &out_action_id)?;
    for conn in connected_nodes_inputs.iter() {
        let node = graph.nodes.get(conn.0).ok_or_else(|| {
            anyhow::anyhow!("Node with id {:?} not found in the graph", conn.0)
                .context("get_next_action_nodes found NodeId, but couldn't get node data")
        })?;
        let input_name: &'a str = node
            .inputs
            .iter()
            .find(|item| item.1 == conn.1)
            .ok_or_else(|| {
                anyhow::anyhow!("Input with id {:?} not found in the node", conn.1)
                    .context("get_next_action_nodes found InputId, but couldn't get input data")
            })
            .map(|e| e.0.as_ref())?;
        res.push((node, input_name));
    }
    Ok(res)
}