use std::borrow::Cow;
use crate::app::types::PulseGraphState;
use super::types::PulseNodeTemplate;

pub fn help_hover_text<'a>(template: PulseNodeTemplate, _user_state: &PulseGraphState) -> Cow<'a, str> {
    match template {
        PulseNodeTemplate::CellPublicMethod => "(Entry point) Exposes a public method to the game as an entity input in case of a ServerPointEntity domain. \
            The input will be named after the method name. an argument can be passed in which is accessible from the argument port.",
        PulseNodeTemplate::EntFire => "Fires an output on an entity either by name, or by handle.",
        PulseNodeTemplate::CellWait => "Pauses current cursor for a given duration.",
        PulseNodeTemplate::GetVar => "Retrive the value under the variable (look at the left side, to add a variable)",
        PulseNodeTemplate::SetVar => "Saves a value under the variable (look at the left side, to add a variable)",
        PulseNodeTemplate::EventHandler => "(Entry point) fires action when a game event occurs. Some events provide additional data.\
            The available events are loaded from the binding file for the current game",
        PulseNodeTemplate::Operation => "Runs a mathmetical operations on two operands, returning a new value. Supported operations:
            + (ADD)
            - (SUB)
            * (MUL)
            / (DIV)
            % (MOD)",
        PulseNodeTemplate::FindEntByName => "Finds an entity by the given targetname and class within the current map.",
        PulseNodeTemplate::DebugWorldText => "Displays a debug text on a given entity. NOTE: this is only visible in singleplayer.",
        PulseNodeTemplate::DebugLog => "Prints something to the console.",
        PulseNodeTemplate::FireOutput => "Fires an entity output from the graph entity which can be handled by the game \
            Look at the left of the window to add custom outputs",
        PulseNodeTemplate::GraphHook => "(Entry point) fires action when game requests the graph to perform an action",
        PulseNodeTemplate::Convert => "Converts between value types. NOTE: Not all conversions are supported",
        PulseNodeTemplate::ForLoop => "A ranged loop with an index, just like in C",
        PulseNodeTemplate::WhileLoop => "A while loop: runs an action until the given condition evaluates as false. \
             Use 'Compare Output' node to feed the condition port.",
        PulseNodeTemplate::StringToEntityName => "Converts a raw string to an entity name. This is needed for some reason?",
        PulseNodeTemplate::InvokeLibraryBinding => "Fires a game function, functions are defined in the bindings file, and can vary between games.",
        PulseNodeTemplate::FindEntitiesWithin => "Find first entity in a given range, starting from a specified entity. \
            It's possible to 'iterate' over entities, by supplying the last found entity handle as the value for pStartEntity",
        PulseNodeTemplate::CompareOutput => "Compares two values, returns a condition, which can be used for 'Compare If', or 'While loop' nodes. \
            Supported operations: ==, !=, <, >, <=, >=",
        PulseNodeTemplate::CompareIf => "Checks if a condition is true and runs the specified action when it is. \
            Use the CompareOutput node to feed the condition port.",
        PulseNodeTemplate::IntSwitch => "Match the provided integer value and run an action based on the chosen value. \
            Use the 'caselabel' input to select the integer value, and click 'Add parameter' to add a output.",
        PulseNodeTemplate::SoundEventStart => "Starts a sound event, coming from an entity, or from the world. Output needs to be connected somewhere. \
            Tip: There's a library function for adjusting such sound events, called 'Sound Event Set Param Float'.",
        PulseNodeTemplate::Function => "A remote node that can be called from multiple places. Put a name into the textbox, to reference it later by CallNode.",
        PulseNodeTemplate::CallNode => "Allows to call remote nodes from anywhere. For example a 'Function'.",
        PulseNodeTemplate::ListenForEntityOutput => "Listens to an output from the provided entity in the current map, causing an action if it gets triggered. Also provides the activator entity handle.",
        PulseNodeTemplate::Timeline => "Runs actions in a sequential order with a delay between each action.",
        PulseNodeTemplate::NewArray => "Creates a new array of the provided type. You can also add initial values if applicable to the type - ones that are producable at compile time. Otherwise manually append elements.",
        // PulseNodeTemplate::LibraryBindingAssigned { binding } => {
        //     let binding = user_state.get_library_binding_from_index(&binding);
        //     if let Some(binding) = binding {
        //         &binding.description.unwrap_or_default()
        //     }
        // }
        _ => "",
    }.into()
}
