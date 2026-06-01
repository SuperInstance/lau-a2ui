# lau-a2ui

> Rendering-agnostic state representation — one PLATO world state, infinite renderings.

## What This Does

This crate defines the core data model for rendering PLATO agent states across any UI backend. Build your state once as an `A2UIState`, then render it as JSON, ASCII art, MUD text, HTML, Unity manifests, Godot scenes, Roblox places, Telegram messages, SSML voice output, or debug dumps. It also supports binary serialization (MessagePack), incremental diffs between states, and a conservation check for energy invariants.

## The Key Idea

Separation of *what* from *how*. The agent builds state in a rendering-agnostic format (`A2UIState`). The `A2UIRenderer` translates that state into whatever the viewer needs — a terminal dashboard, a Unity scene, a Telegram chat message, or spoken SSML. The state includes agents, rooms, hardware, bridges, intentions, a "vibe field" for ambient energy, and an event stream. Diffs (`A2UIDiff`) let you send incremental updates instead of full state redraws.

## Install

```bash
cargo add lau-a2ui
```

## Quick Start

```rust
use lau_a2ui::*;

// Build state
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

// Render to any backend
let json_out = state.render(RendererType::JSON);
let html_out = state.render(RendererType::Web);
let mud_out = state.render(RendererType::MUD);
let telegram_out = state.render(RendererType::Telegram);
let voice_out = state.render(RendererType::Voice);

// Serialize to binary
let bytes = state.serialize();
let restored = A2UIState::deserialize(&bytes).unwrap();

// Compute incremental diff
let diff = state.diff(&previous_state);
if !diff.is_empty() {
    let diff_bytes = diff.serialize();
}
```

## API Reference

### State Types

| Type | Description |
|------|-------------|
| `A2UIState` | Top-level rendering-agnostic state. Contains `world`, `agents`, `rooms`, `hardware`, `bridges`, `intentions`, `field`, `events`. |
| `WorldState` | World metadata: `name`, `bounds`, `energy_total`, `tick_rate`, `ambient_light`, `ambient_sound`. |
| `AgentState` | Agent render state: `agent_id`, `name`, `archetype`, `position`, `rotation`, `energy`, `level`, `mood`, `current_task`, `visible`, `voice_hint`. |
| `RoomState` | Room render state: `room_id`, `name`, `room_type`, `position`, `size`, `energy_level`, `temperature`, `occupants`, `exits`, `contents`. |
| `HardwareState` | Device state: `hardware_id`, `name`, `hw_type`, `position`, `connected`, `active`, `last_reading`, `status_color`. |
| `BridgeState` | Portal state: `bridge_id`, `target`, `status`, `position`, `active`, `color`. |
| `IntentionState` | Goal tracking: `intention_id`, `goal`, `status`, `progress`, `energy_allocated`, `assigned_agent`. |
| `FieldState` | Vibe field: `resolution`, `width`, `height`, `samples`, `hotspots`, `gradient_direction`. |

### Enums

| Type | Description |
|------|-------------|
| `AgentMood` | `Focused`, `Relaxed`, `Alert`, `Confused`, `Celebrating` |
| `RendererType` | `Unity`, `Godot`, `Roblox`, `Web`, `Telegram`, `Voice`, `MUD`, `JSON`, `ASCII`, `Debug` |
| `Event` | `AgentMoved`, `RoomCreated`, `EnergyDeposited`, `IntentionCompleted`, `OverrideTriggered`, `CrewPromoted`, `BridgeOpened`, `Alert` |

### Core Methods

#### `A2UIState`
- `new() -> Self` — Create empty state.
- `serialize(&self) -> Vec<u8>` — MessagePack binary encoding.
- `deserialize(data: &[u8]) -> Result<Self, String>` — MessagePack decoding.
- `render(&self, renderer: RendererType) -> RenderedOutput` — Dispatch to backend renderer.
- `diff(&self, previous: &A2UIState) -> A2UIDiff` — Compute incremental changes.
- `is_conserved(&self) -> bool` — Check `energy_total >= 0.0`.

#### `A2UIDiff`
- `is_empty(&self) -> bool` — True if no agents/rooms changed, no events, zero energy delta.
- `serialize(&self) -> Vec<u8>` / `deserialize(data) -> Result<Self, String>` — MessagePack round-trip.

#### `A2UIRenderer` (static methods)
- `render_json`, `render_mud`, `render_ascii`, `render_unity_manifest`, `render_godot_scene`, `render_roblox_place`, `render_web`, `render_telegram`, `render_voice`, `render_debug` — Individual backend renderers.

#### `RenderedOutput`
- `as_text(&self) -> Option<&str>` — Get UTF-8 text content.
- `as_json(&self) -> Option<&str>` — Get JSON if content type matches.

### Rendering Backends

| Backend | Output | Use Case |
|---------|--------|----------|
| `JSON` | Pretty-printed JSON | APIs, debugging |
| `MUD` | Text adventure format | Terminal games |
| `ASCII` | Character grid | Terminal dashboards |
| `Unity` | Unity manifest JSON | Unity game engine |
| `Godot` | Godot scene JSON | Godot game engine |
| `Roblox` | Roblox place JSON | Roblox platform |
| `Web` | Full HTML page | Browser dashboards |
| `Telegram` | HTML with emoji | Telegram bot messages |
| `Voice` | SSML XML | Text-to-speech |
| `Debug` | Debug fmt output | Development |

## How It Works

State is a flat collection of typed entities. Each renderer walks the state and produces backend-specific output:
- **MUD renderer** finds the viewer's room and describes exits, contents, and other occupants
- **ASCII renderer** maps positions onto a character grid
- **Unity/Godot/Roblox renderers** produce JSON scene manifests
- **Web renderer** generates a styled HTML page with tables
- **Telegram renderer** produces emoji-rich HTML with mood indicators
- **Voice renderer** generates SSML for text-to-speech

The diff system compares two states by ID: agents/rooms appearing, disappearing, or changing produce `A2UIDiff` entries.

## Testing

**55+ tests** covering:
- State construction and defaults
- MessagePack serialization round-trips
- Diff computation (add, remove, update agents/rooms, energy delta)
- All 10 renderer backends produce non-empty output
- MUD renderer shows room details, exits, contents, occupants
- ASCII renderer places agents/rooms correctly, hides invisible agents
- Individual type serde round-trips (WorldState, AgentMood, FieldState, etc.)

## License

MIT
