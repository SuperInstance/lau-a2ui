# lau-a2ui

> Rendering-agnostic state representation — one PLATO world state, infinite renderings.

## What This Does

This crate defines the core data model for rendering PLATO agent states across **any** UI backend. Build your state once as an `A2UIState`, then render it as:

- **JSON** — pretty-printed API output
- **ASCII** — character grid for terminal dashboards
- **MUD** — text adventure format with rooms, exits, and occupants
- **Web** — styled HTML page with tables
- **Unity** — scene manifest JSON for the Unity game engine
- **Godot** — scene node tree JSON for Godot
- **Roblox** — workspace place JSON for Roblox
- **Telegram** — emoji-rich HTML messages for Telegram bots
- **Voice** — SSML XML for text-to-speech engines
- **Debug** — full `Debug` format dump

It also provides **binary serialization** (MessagePack), **incremental diffs** between states, and a **conservation check** for energy invariants.

## The Key Idea

Separation of *what* from *how*. The agent builds state in a rendering-agnostic format (`A2UIState`). The `A2UIRenderer` translates that state into whatever the viewer needs — a terminal dashboard, a Unity scene, a Telegram chat message, or spoken SSML. The state includes agents, rooms, hardware, bridges, intentions, a "vibe field" for ambient energy, and an event stream. Diffs (`A2UIDiff`) let you send incremental updates instead of full state redraws.

```
                    ┌────────────────┐
                    │   A2UIState    │
                    │ (agnostic)     │
                    └───────┬────────┘
                            │
              ┌─────────────┼──────────────┐
              │ render()    │ diff()        │ serialize()
              ▼             ▼               ▼
     ┌─────────────┐  ┌──────────┐   ┌──────────┐
     │ A2UIRenderer│  │ A2UIDiff │   │ MsgPack  │
     └──┬──┬──┬───┘  └──────────┘   └──────────┘
        │  │  │
   ┌────┘  │  └────┐
   ▼       ▼       ▼
 JSON    MUD    Unity/Godot/Roblox/Web/Telegram/Voice/Debug
```

## Install

```bash
cargo add lau-a2ui
```

**Dependencies:** `serde`, `serde_json`, `rmp-serde` (MessagePack)

## Quick Start

```rust
use lau_a2ui::*;

// 1. Build state
let mut state = A2UIState::new();
state.world.name = "MyWorld".into();
state.tick = 42;
state.agents.push(AgentState {
    agent_id: "a1".into(),
    name: "Alpha".into(),
    archetype: "scout".into(),
    position: (10.0, 20.0, 0.0),
    rotation: (0.0, 0.0, 0.0),
    energy: 100.0,
    level: 3,
    mood: AgentMood::Focused,
    current_task: Some("exploring".into()),
    visible: true,
    voice_hint: None,
});
state.rooms.push(RoomState {
    room_id: "hub".into(),
    name: "Hub".into(),
    room_type: "central".into(),
    position: (0.0, 0.0, 0.0),
    size: (20.0, 20.0, 5.0),
    energy_level: 50.0,
    temperature: 22.0,
    occupants: vec!["a1".into()],
    exits: vec![Exit {
        target_room: "north-wing".into(),
        direction: "north".into(),
        locked: false,
    }],
    contents: vec!["toolbox".into()],
});

// 2. Render to any backend
let json_out = state.render(RendererType::JSON);
let html_out = state.render(RendererType::Web);
let mud_out = state.render(RendererType::MUD);
let telegram_out = state.render(RendererType::Telegram);
let voice_out = state.render(RendererType::Voice);

// 3. Serialize to binary (MessagePack)
let bytes = state.serialize();
let restored = A2UIState::deserialize(&bytes).unwrap();

// 4. Compute incremental diff
let diff = state.diff(&previous_state);
if !diff.is_empty() {
    let diff_bytes = diff.serialize();  // send only changes
}
```

## API Reference

### State Types

| Type | Description |
|------|-------------|
| `A2UIState` | Top-level rendering-agnostic state. Contains `version`, `timestamp`, `tick`, `world`, `agents`, `rooms`, `hardware`, `bridges`, `intentions`, `field`, `events`. |
| `WorldState` | World metadata: `name`, `bounds` (3D tuple), `energy_total`, `tick_rate`, `ambient_light`, `ambient_sound`. |
| `AgentState` | Agent render state: `agent_id`, `name`, `archetype`, `position` (3D), `rotation` (3D), `energy`, `level`, `mood`, `current_task`, `visible`, `voice_hint`. |
| `RoomState` | Room render state: `room_id`, `name`, `room_type`, `position` (3D), `size` (3D), `energy_level`, `temperature`, `occupants`, `exits`, `contents`. |
| `Exit` | Room exit: `target_room`, `direction`, `locked`. |
| `HardwareState` | Device state: `hardware_id`, `name`, `hw_type`, `position`, `connected`, `active`, `last_reading`, `status_color`. |
| `BridgeState` | Portal state: `bridge_id`, `target`, `status`, `position`, `active`, `color`. |
| `IntentionState` | Goal tracking: `intention_id`, `goal`, `status`, `progress`, `energy_allocated`, `assigned_agent`. |
| `FieldState` | Vibe field: `resolution`, `width`, `height`, `samples`, `hotspots`, `gradient_direction`. |
| `RenderedOutput` | Rendered result: `renderer`, `content_type`, `data`, `metadata`. |
| `A2UIDiff` | Incremental state change: `added_agents`, `removed_agents`, `added_rooms`, `removed_rooms`, `updated_agents`, `updated_rooms`, `events`, `energy_delta`. |

### Enums

| Type | Variants |
|------|----------|
| `AgentMood` | `Focused`, `Relaxed`, `Alert`, `Confused`, `Celebrating` |
| `RendererType` | `Unity`, `Godot`, `Roblox`, `Web`, `Telegram`, `Voice`, `MUD`, `JSON`, `ASCII`, `Debug` |
| `Event` | `AgentMoved`, `RoomCreated`, `EnergyDeposited`, `IntentionCompleted`, `OverrideTriggered`, `CrewPromoted`, `BridgeOpened`, `Alert` |

### Core Methods

#### `A2UIState`
- `new() -> Self` — Create empty state with sensible defaults.
- `serialize(&self) -> Vec<u8>` — MessagePack binary encoding.
- `deserialize(data: &[u8]) -> Result<Self, String>` — MessagePack decoding.
- `render(&self, renderer: RendererType) -> RenderedOutput` — Dispatch to a backend renderer.
- `diff(&self, previous: &A2UIState) -> A2UIDiff` — Compute incremental changes (agents/rooms added/removed/updated, events, energy delta).
- `is_conserved(&self) -> bool` — Check `energy_total >= 0.0`.

#### `A2UIDiff`
- `is_empty(&self) -> bool` — True if no agents/rooms changed, no events, and zero energy delta.
- `serialize(&self) -> Vec<u8>` / `deserialize(data) -> Result<Self, String>` — MessagePack round-trip.

#### `A2UIRenderer` (static methods)
Each takes `(&A2UIState, ...)` and returns `RenderedOutput`:
- `render_json` — Pretty-printed JSON with version/tick metadata
- `render_mud(state, viewer_agent)` — Text adventure: room description, exits, contents, occupants
- `render_ascii` — Character grid with agents/rooms/bridges mapped to characters
- `render_unity_manifest` — Unity scene manifest JSON
- `render_godot_scene` — Godot node tree JSON
- `render_roblox_place` — Roblox workspace JSON
- `render_web` — Full styled HTML page with tables
- `render_telegram(state, viewer_agent)` — HTML with emoji mood indicators and viewer marker (▶)
- `render_voice(state, viewer_agent)` — SSML for text-to-speech
- `render_debug` — Rust `Debug` format dump

#### `RenderedOutput`
- `as_text(&self) -> Option<&str>` — Get UTF-8 text content (None if binary).
- `as_json(&self) -> Option<&str>` — Get JSON string (None if content type doesn't contain "json").

### Rendering Backends

| Backend | Output Format | Use Case |
|---------|--------------|----------|
| `JSON` | Pretty-printed JSON | APIs, debugging, serialization |
| `MUD` | Text adventure format | Terminal games, immersive UIs |
| `ASCII` | Character grid | Terminal dashboards, low-bandwidth |
| `Unity` | Unity manifest JSON | Unity game engine integration |
| `Godot` | Godot scene JSON | Godot game engine integration |
| `Roblox` | Roblox place JSON | Roblox platform |
| `Web` | Full HTML page with CSS | Browser dashboards |
| `Telegram` | HTML with emoji | Telegram bot messages |
| `Voice` | SSML XML | Text-to-speech systems |
| `Debug` | Debug fmt output | Development and debugging |

## How It Works

### State Model
State is a flat collection of typed entities — agents, rooms, hardware, bridges, intentions, and a scalar field. All positions and sizes are 3D tuples `(f64, f64, f64)`. The `FieldState` represents a sampled scalar "vibe" field with configurable resolution, hotspot locations, and gradient direction.

### Rendering Pipeline
Each renderer walks the state and produces backend-specific output:

- **MUD renderer**: Finds the viewer's current room by matching agent ID to room occupants. Describes exits (showing "locked" for locked exits), contents ("You see: ..."), and other agents in the room ("Also here: ..."). If no viewer is specified, lists all rooms with occupant counts.
- **ASCII renderer**: Maps world positions onto a character grid. Rooms become `#`, agents become their first initial (uppercase), active bridges become `@`, inactive bridges become `O`. Invisible agents are hidden.
- **Unity/Godot/Roblox renderers**: Produce structured JSON manifests with engine-specific field names (e.g., `CFrame` for Roblox, `transform` for Godot).
- **Web renderer**: Generates a complete HTML document with inline CSS (dark theme, monospace font) and data tables for agents and rooms.
- **Telegram renderer**: Produces HTML with emoji mood indicators (🎯 Focused, 😌 Relaxed, ⚠️ Alert, ❓ Confused, 🎉 Celebrating) and marks the viewer with ▶.
- **Voice renderer**: Generates SSML `<speak>` blocks describing the viewer's task, room, and other occupants.

### Diff System
The diff system compares two states by matching entity IDs:
1. **Added**: entities present in current but not in previous
2. **Removed**: entities present in previous but not in current
3. **Updated**: entities present in both but with different data
4. **Events**: current state's event stream is copied to the diff
5. **Energy delta**: `current.world.energy_total - previous.world.energy_total`

A diff is considered "empty" only when all lists are empty *and* the energy delta is within `f64::EPSILON` of zero.

## The Math

### Energy Conservation
The `is_conserved()` check enforces the invariant `energy_total ≥ 0`, a simple conservation law for the simulation. The diff's `energy_delta` tracks how energy changes between ticks.

### Scalar Field Sampling
The `FieldState` represents a discretized scalar field over the world:
- `resolution`: samples per unit area
- `width × height`: spatial extent
- `samples`: raw scalar values
- `hotspots`: local maxima as `(x, y, value)` tuples
- `gradient_direction`: `(dx, dy)` indicating the overall field gradient

### Diff as Set Operations
The diff computation is essentially set operations on entity IDs:
```
added   = current_ids \ previous_ids
removed = previous_ids \ current_ids
updated = {id | id ∈ current_ids ∩ previous_ids ∧ data(id) changed}
```

## Testing

**67 tests** covering:
- State construction, defaults, and `Default` trait equivalence
- MessagePack serialization round-trips (full state and empty state)
- Deserialization of garbage and empty input
- Diff computation: add/remove/update agents and rooms, energy delta, event copying
- Diff `is_empty()` for all edge cases (with/without events, with/without energy delta)
- Diff serialization round-trip
- All 10 renderer backends produce non-empty output on both full and empty states
- JSON renderer: metadata correctness, round-trip through serde
- MUD renderer: viewer room detection, exits, contents, occupants, locked exits
- ASCII renderer: agent/room placement, invisible agent hiding
- Unity/Godot/Roblox renderers: JSON structure validation
- Web renderer: HTML structure, agent/room inclusion
- Telegram renderer: mood emoji, viewer marker
- Voice renderer: SSML structure, task inclusion
- RenderedOutput helper methods (`as_text`, `as_json`)
- Individual type serde round-trips: WorldState, AgentMood, RendererType, FieldState, HardwareState, BridgeState, IntentionState, Exit
- Event text formatting for all 8 event variants

```bash
cargo test    # Run all 67 tests
```

## License

MIT
