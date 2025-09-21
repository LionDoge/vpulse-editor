# An unofficial Pulse graph editor
"Pulse" is an in-development system for visual scripting in newer Source 2 engine games that is. Dota 2, CS2, and Deadlock right now.
There's no official tooling for creating these files, so this tool is supposed to serve as a temporary solution. As you will soon find out it's not perfect, and I didn't really intend it to be, as it's meant to be used as experimentation.

This tool is intended to work for Counter-Strike 2, as other games have barely any use for Pulse graphs. This might change in the future.
This tool has only been tested on Windows 10 and 11 currently, support for Linux might come in the future.

> [!NOTE]
> The tool and the documentation provided below assumes decent understanding of creating maps and/or modding Counter-Strike 2 or other Source 2 games, not everything is very well documented, as things are likely to change often. The tool is not perfect, and game updates may, **and probably will** break existing compiled graphs, which will be required to be compiled by a newer version of this software once available. It is probably a good idea to not be strongly dependent on systems that are still in development, if you can't handle possible breakages in the future.

# Game setup
Some initial setup inside the game files is required to make working with Pulse graphs way easier.

**NOTE:** These files don't interfer with VAC, it's possible to play normally with these modifications. Also, these modifications might disappear after game updates, and certainly will disappear when verifying game files!

## FGD setup
Go to `game/csgo/csgo.fgd` and add these lines, somewhere after all the imports and exports and all that..
```
@PointClass base(Targetname) tags( Logic ) iconsprite("editor/point_pulse.vmat") = point_pulse : "An entity that acts as a container for pulse graphs"
[
    graph_def(string) : "Graph path" : "" :
]
```
## assettypes setup
Go to `game/bin/assettypes_common.txt` and add these lines in between other asset definitons.
```
pulse_graph = 
{
	_class = "CResourceAssetTypeInfo"
	m_FriendlyName = "Pulse Graph"
	m_Ext = "vpulse"
	m_IconLg = "game:tools/images/assettypes/javascript_lg.png"
	m_IconSm = "game:tools/images/assettypes/javascript_sm.png"
	m_CompilerIdentifier = "CompileKV3"
	m_Blocks = 
	[
		{
			m_BlockID = "DATA"
			m_Encoding = "RESOURCE_ENCODING_KV3"
		},
	]
}
```
This allows for Pulse graphs to be compiled properly, and to be displayed in the asset browser.

## Manual compile
You need to have rust tooling installed first. Use cargo to build the tool with `cargo build --release`. Once finished it should be output to 'target/release/pulseedit.exe' directory, you'll only need the executable, and the bindings file ('bindings_cs2.json') next to the executable.

## Pre-built release
Download the newest version from [releases](https://github.com/LionDoge/vpulse-editor/releases). It includes almost everything needed to run the tool. Once unpacked, just run the pulseedit executable.

# Basic Usage
> [!NOTE]
> This section will see more additions as time goes on, and as possibly more questions appear.

<img src="reference_img/img1.png" alt="drawing" width="600"/>

- On the right side there's the main viewport, right click to open the menu for adding new nodes. 
- On the left side you can add variables that can be used by graphs to remember information by using Load Variable and Save Variable nodes.
- The top bar allows you to open, save, and compile graphs. 
- In order to compile a graph it needs to be saved first. Point the save file inside your addon's content directory e.g `content/csgo_addons/ADDONNAME/scripts/vscripts/` **NOTE:** The workshop uploader is configured to include only specific directories. Before you upload the map, please verify that the vpulse files you used are properly included when uploading.

## Basic logic flow
In order for a graph to be compilable it must have an entry point, which is a way of exposing the graph's functions to the game. You can find entry points in the 'Inflow' category. For example the 'Public Method' adds a custom entity input on the graph's entity, which can be triggered from a map.

Action nodes, for example EntFire, DebugLog and many others need their `ActionIn` parameter to be connected in order to run. Actions connected like that are ran sequentially, see example below:

![alt text](reference_img/img2.png)

One continous flow from one entry point is refered to as a 'Chunk'. It's good to remeber that for later!

## Using the graph in a map
First make sure you have saved the file inside the approperiate content directory for your addon (explained above) and that the graph is compiled (click 'Compile' in top left of the window). If no errors appear then the compile was succesful.

Add a `point_pulse` entity to your map (make sure that FGD is setup properly!). Also make sure to give it a name for easier testing. The entity needs to be refered to the proper script path i.e. where the graph file is saved and compiled. The file path is relative to the current addon directory. If you have a pulse file in `content/csgo_addons/ADDONNAME/scripts/vscripts/`. You need to input `scripts/vscripts/graph.vpulse` in the 'Graph path' key - Note the .vpulse extension here.

Now compile the map. If you replicated the example pictured above, you should be able to use `ent_fire point_pulse_name Method` to run the method. You should see a console output from the debug logs. You can now modify the graph further and apply the changes instantly by clicking 'Compile'. The game will reload the graph automatically introducing new changes. You can also interact with the graph by using regular entity I/O system. Note however that Hammer will not recognize the input names created within the graph, and will color them red, it doesn't matter and it still should work in the game.

## Value nodes
Not all nodes are action nodes, some just provide values to feed in the input ports and are resolved before the action they're connected to is ran. There are many utility nodes, including ones that grab data from the game, or do operations on values. Refer to the image below for a visual explaination.

![](reference_img/img3.png)

This was a basic explaination on how the graphs are processed. Take a look at examples in the 'examples' directory. You can also hover over the information sign on added nodes in the app to display their usage notes.

## What to avoid
There are some caveats due to how this tool was made. It's not perfect, and there are some situations that are unchecked but are invalid. Refer to the image below.

**Don't connect outputs from the same node between different chunk, or during a split of a conditional (like an if condition). These nodes are processed and then reused, but not every situation is checked, and instead of the nodes being recomputed for each chunk or conditional split, they're reused, resulting in incorrect logic.** If in doubt just make a new node. Refer to the example below for such **invalid** uses.
![](reference_img/img4.png)

# Examples
Examples can be found in the 'examples' directory.

- `inputs` basic introduction to using public methods 
    - 'RunMe' method for a very basic print
    - 'DebugText string' method that accepts a string argument for what to display on a entity named 'dynprop' that is a *prop_dynamic* (needs to be setup properly in Hammer.)
    - 'OnRoundStart' event that prints the current round number to the console when a new round starts.

- `entities` Shows basic interaction with entities: Entity handles, and using EntFire.
    - 'GetHeightAboveWorld' will print height above the world of a entity named 'btn' that is a *func_button*
    - 'ReColor entity_name' will tint all entities to red that are named same as the provided argument.

- `forloop` Shows an example of a nested indexed loop

- `entities2` is a more complex example of finding all entities by a classname. It uses the *Find entities within* node, and a while loop. Each iteration a found entity is saved into a variable and then fed back as the starting entity for the node to find another entity, till all entities are exhausted. In this case the distance between each *decoy_projectile* and a prop named 'src' of *prop_dynamic* will be shown above each decoy when the method 'DisplayDistance' is ran.

- `timing` shows some examples of delaying exeuction

- `remote_nodes_listen_entity_output` Shows examples of so called 'remote nodes' that can be reused. Think like a function in a programming language. It also shows an example of listening to an output from an entity and using the activator handle. In this scenario pressing a *func_button* named `btn` will deal 10 damage to the activator once the listening starts after firing 'StartButtonListen'.

- `radio` Is an example of a 'radio' that can switch between different music with the 'NextSong' method, it also demonstrates how to play sounds, and change their parameters while they're playing. In this 'VolumeUp' and 'VolumeDown' inputs can be used to adjust the volume of a playing song. This is also a good demonstration of how to use operations and conditional checks. NOTE: To hear the songs you need to turn up 'Main menu music volume' in the game's settings, since the example songs are used from music kits.

## Why make this?
Pulse is an in=dev system, suggesting that it could still undergo changes wasting the effort of writing this app. The release of an official editor would also make this useless, so why bother spending time on it? Well, I like challenge and reverse engineering components of software, not only that, but I noticed that Pulse could do much more than any other possible scripting/map making methods. Granted, it still doesn't open up that much possibilities, but at least some more than was possible before, so this project is not totally useless! This project was also started as part of my Rust programmiourse at university, the only other ideas I had were ones that would be one and done, so I decided to pick something that I could also continue. So here we are! All of this wng cas possible due to my weird obsessions.