# Controls

The **sandbox** uses the 1.12 client's own third-person controls. There is no menu: the game boots
straight into the world with your character.

| Input | Action |
|---|---|
| **W** / **S** (or **Up** / **Down**) | run forward / backpedal |
| **A** / **D** (or **Left** / **Right**) | turn; **strafe** instead while the right mouse button is held |
| **Q** / **E** | strafe left / right |
| **Space** | jump |
| **Right mouse**, drag | steer: the character turns with the camera (the cursor hides and locks) |
| **Left mouse**, drag | orbit the camera around the character; it swings back behind you as you move |
| **Both mouse buttons** | run forward, steering with the mouse |
| **Mouse wheel** | zoom, from first person out to 30 yards |
| **Num Lock** | autorun |
| **X** / **Z** | sit or stand / sheathe or draw the weapon |
| **F1** | hide or show the help line and the coordinates readout |

Swimming, falling and stepping up a ledge work as in the client. Walking into a building collides
with it.

Choose who and where with the page address, for example
`http://127.0.0.1:8090/?race=orc&class=warrior&map=1&x=-950&y=-2700`. The parameters are in
[docs/SANDBOX.md](docs/SANDBOX.md).

On Linux, Chrome needs `--enable-unsafe-webgpu` (or `chrome://flags/#enable-unsafe-webgpu`) for the
world to render.
