use std::borrow::Cow;
use crate::app::types::PulseGraphState;
use super::types::PulseNodeTemplate;

pub fn help_hover_text<'a>(template: PulseNodeTemplate, user_state: &'a PulseGraphState) -> Cow<'a, str> {
    match template {
        PulseNodeTemplate::CellPublicMethod => "(Entry point) Exposes a public method to the game as an entity input in case of a ServerPointEntity domain. \
            The input will be named after the method name. an argument can be passed in which is accessible from the argument port.".into(),
        PulseNodeTemplate::EntFire => "Fires an output on an entity either by name, or by handle.".into(),
        PulseNodeTemplate::CellWait => "Pauses current cursor for a given duration.".into(),
        PulseNodeTemplate::GetVar => "Retrive the value under the variable (look at the left side, to add a variable)".into(),
        PulseNodeTemplate::SetVar => "Saves a value under the variable (look at the left side, to add a variable)".into(),
        PulseNodeTemplate::EventHandler => "(Entry point) fires action when a game event occurs. Some events provide additional data.\
            The available events are loaded from the binding file for the current game".into(),
        PulseNodeTemplate::Operation => "Runs a logical operation on two operands, returning a new value. Supported operations:
            + (ADD)
            - (SUB)
            * (MUL)
            / (DIV)
            % (MOD)".into(),
        PulseNodeTemplate::FindEntByName => "Finds an entity by the given targetname and class within the current map.".into(),
        PulseNodeTemplate::DebugWorldText => "Displays a debug text on a given entity. Works only in development environment.".into(),
        PulseNodeTemplate::DebugLog => "Prints something to the console.".into(),
        PulseNodeTemplate::FireOutput => "Fires an entity output from the graph entity which can be handled by the game \
            Look at the left of the window to add custom outputs".into(),
        PulseNodeTemplate::GraphHook => "(Entry point) fires action when game requests the graph to perform an action".into(),
        PulseNodeTemplate::Convert => "Converts between value types. NOTE: Not all conversions are supported".into(),
        PulseNodeTemplate::ForLoop => "A ranged loop with an index, the range is inclusive of the 'to' value.".into(),
        PulseNodeTemplate::WhileLoop => "A while loop: runs an action until the given condition evaluates as false. \
             Use 'Compare Output' node to feed the condition port.".into(),
        PulseNodeTemplate::StringToEntityName => "Converts a raw string to an entity name. This is needed for some reason?".into(),
        PulseNodeTemplate::InvokeLibraryBinding => "Fires a game function, functions are defined in the bindings file, and can vary between games.".into(),
        PulseNodeTemplate::FindEntitiesWithin => "Find first entity in a given range, starting from a specified entity. \
            It's possible to 'iterate' over entities, by supplying the last found entity handle as the value for pStartEntity".into(),
        PulseNodeTemplate::CompareOutput => "Compares two values, returns a condition, which can be used for nodes that require conditions i.e 'If' \
            Supported operations: ==, !=, <, >, <=, >=. Note that string comparison here is case-insensitive. There exists a separate node for case-sensitive comparison.".into(),
        PulseNodeTemplate::CompareIf => "Checks if a condition is true and runs the specified action when it is. \
            Use the CompareOutput node to feed the condition port.".into(),
        PulseNodeTemplate::IntSwitch => "Match the provided integer value and run an action based on the chosen value. \
            Use the 'caselabel' input to select the integer value, and click 'Add parameter' to add a output.".into(),
        PulseNodeTemplate::SoundEventStart => "Starts a sound event, coming from an entity, or from the world. Output needs to be connected somewhere. \
            Tip: There's a library function for adjusting such sound events, called 'Sound Event Set Param Float'.".into(),
        PulseNodeTemplate::Function => "A remote node that can be called from multiple places. Put a name into the textbox, to reference it later by CallNode.".into(),
        PulseNodeTemplate::CallNode => "Allows to call remote nodes from anywhere. For example a 'Function'.".into(),
        PulseNodeTemplate::ListenForEntityOutput => "Listens to an output from the provided entity in the current map, causing an action if it gets triggered. Also provides the activator entity handle.".into(),
        PulseNodeTemplate::Timeline => "Runs actions in a sequential order with a delay between each action.".into(),
        PulseNodeTemplate::NewArray => "Creates a new array of the provided type. You can also add initial values if applicable to the type, otherwise they may be added later at runtime.".into(),
        PulseNodeTemplate::LibraryBindingAssigned { binding } => {
            user_state
                .get_library_binding_from_index(binding)
                .and_then(|b| b.description.as_ref())
                .map(|d| d.as_str().into())
                .unwrap_or_else(|| "".into())
        }
        _ => "".into(),
    }
}
