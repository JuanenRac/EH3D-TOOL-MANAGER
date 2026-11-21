# EH3D-TOOL-MANAGER: not yet

Good ideas that are not in the current version. Writing them here keeps anybody from implementing them early or twice.

| Idea | Why not yet | Revisit when |
|---|---|---|
| The CAM programs reading the crib | Only ISOCAM reads the CSV table; the others keep their own tool tables | the programs that cut agree on the crib as their source of tools |
| The 2D profile of each tool | The crib holds the sizes of a tool, not its shape | a CAM program needs the shape for its paths |
| A file dialog in the window | The window asks for the file as a written path | the window has been looked at on a screen of each system |
| A database for the crib | A JSON file is enough for one shop and one person at a time; the server keeps revisions for the rest | several programs write the crib at the same time |
| Feeds and speeds checked on a machine | The numbers are typical values and nothing here has run on a machine | a person cuts with a tool and gives the result |
| Measuring a tool | The values are typed; nothing reads a real tool | the vision program of the suite measures a tool |
