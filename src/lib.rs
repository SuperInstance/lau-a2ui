//! # lau-a2ui — Agent-to-UI Rendering Protocol
//!
//! Rendering-system-agnostic state representation. One PLATO state, infinite renderings.
//! The agent builds in A2A-native; the human sees whatever rendering they prefer.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// 1. Core enums
// ---------------------------------------------------------------------------

/// Agent mood for rendering hints.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentMood {
    Focused,
    Relaxed,
    Alert,
    Confused,
    Celebrating,
}

/// Renderer type — which backend to target.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RendererType {
    Unity,
    Godot,
    Roblox,
    Web,
    Telegram,
    Voice,
    MUD,
    JSON,
    ASCII,
    Debug,
}

/// An event that happened (for animation / dialog).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Event {
    AgentMoved {
        agent: String,
        from: (f64, f64, f64),
        to: (f64, f64, f64),
    },
    RoomCreated {
        room: String,
    },
    EnergyDeposited {
        location: (f64, f64, f64),
        amount: f64,
    },
    IntentionCompleted {
        intention: String,
        success: bool,
    },
    OverrideTriggered {
        by: String,
    },
    CrewPromoted {
        archetype: String,
        new_level: u32,
    },
    BridgeOpened {
        target: String,
    },
    Alert {
        message: String,
        severity: String,
    },
}

// ---------------------------------------------------------------------------
// 2. State structs
// ---------------------------------------------------------------------------

/// World-level state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorldState {
    pub name: String,
    pub bounds: (f64, f64, f64),
    pub energy_total: f64,
    pub tick_rate: f64,
    pub ambient_light: f64,
    pub ambient_sound: f64,
}

/// An agent's renderable state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentState {
    pub agent_id: String,
    pub name: String,
    pub archetype: String,
    pub position: (f64, f64, f64),
    pub rotation: (f64, f64, f64),
    pub energy: f64,
    pub level: u32,
    pub mood: AgentMood,
    pub current_task: Option<String>,
    pub visible: bool,
    pub voice_hint: Option<String>,
}

/// An exit from a room.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Exit {
    pub target_room: String,
    pub direction: String,
    pub locked: bool,
}

/// A room's renderable state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoomState {
    pub room_id: String,
    pub name: String,
    pub room_type: String,
    pub position: (f64, f64, f64),
    pub size: (f64, f64, f64),
    pub energy_level: f64,
    pub temperature: f64,
    pub occupants: Vec<String>,
    pub exits: Vec<Exit>,
    pub contents: Vec<String>,
}

/// Hardware renderable state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HardwareState {
    pub hardware_id: String,
    pub name: String,
    pub hw_type: String,
    pub position: (f64, f64, f64),
    pub connected: bool,
    pub active: bool,
    pub last_reading: Option<f64>,
    pub status_color: String,
}

/// Bridge renderable state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BridgeState {
    pub bridge_id: String,
    pub target: String,
    pub status: String,
    pub position: (f64, f64, f64),
    pub active: bool,
    pub color: String,
}

/// Intention renderable state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IntentionState {
    pub intention_id: String,
    pub goal: String,
    pub status: String,
    pub progress: f64,
    pub energy_allocated: f64,
    pub assigned_agent: Option<String>,
}

/// Vibe field samples for rendering.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldState {
    pub resolution: usize,
    pub width: usize,
    pub height: usize,
    pub samples: Vec<f64>,
    pub hotspots: Vec<(f64, f64, f64)>,
    pub gradient_direction: (f64, f64),
}

// ---------------------------------------------------------------------------
// 3. Rendered output
// ---------------------------------------------------------------------------

/// The result of rendering state through a backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderedOutput {
    pub renderer: RendererType,
    pub content_type: String,
    pub data: Vec<u8>,
    pub metadata: HashMap<String, String>,
}

impl RenderedOutput {
    pub fn as_text(&self) -> Option<&str> {
        std::str::from_utf8(&self.data).ok()
    }

    pub fn as_json(&self) -> Option<&str> {
        if self.content_type.contains("json") {
            std::str::from_utf8(&self.data).ok()
        } else {
            None
        }
    }
}

// ---------------------------------------------------------------------------
// 4. A2UIDiff — incremental state change
// ---------------------------------------------------------------------------

/// What changed between two states.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct A2UIDiff {
    pub added_agents: Vec<AgentState>,
    pub removed_agents: Vec<String>,
    pub added_rooms: Vec<RoomState>,
    pub removed_rooms: Vec<String>,
    pub updated_agents: Vec<(String, AgentState)>,
    pub updated_rooms: Vec<(String, RoomState)>,
    pub events: Vec<Event>,
    pub energy_delta: f64,
}

impl A2UIDiff {
    pub fn is_empty(&self) -> bool {
        self.added_agents.is_empty()
            && self.removed_agents.is_empty()
            && self.added_rooms.is_empty()
            && self.removed_rooms.is_empty()
            && self.updated_agents.is_empty()
            && self.updated_rooms.is_empty()
            && self.events.is_empty()
            && self.energy_delta.abs() < f64::EPSILON
    }

    pub fn serialize(&self) -> Vec<u8> {
        rmp_serde::to_vec(self).unwrap_or_default()
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, String> {
        rmp_serde::from_slice(data).map_err(|e| e.to_string())
    }
}

// ---------------------------------------------------------------------------
// 5. A2UIState — THE rendering-agnostic state
// ---------------------------------------------------------------------------

/// The top-level rendering-agnostic state. One state, infinite renderings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct A2UIState {
    pub version: String,
    pub timestamp: u64,
    pub tick: u64,
    pub world: WorldState,
    pub agents: Vec<AgentState>,
    pub rooms: Vec<RoomState>,
    pub hardware: Vec<HardwareState>,
    pub bridges: Vec<BridgeState>,
    pub intentions: Vec<IntentionState>,
    pub field: FieldState,
    pub events: Vec<Event>,
}

impl A2UIState {
    pub fn new() -> Self {
        Self {
            version: "0.1.0".into(),
            timestamp: 0,
            tick: 0,
            world: WorldState {
                name: String::new(),
                bounds: (0.0, 0.0, 0.0),
                energy_total: 0.0,
                tick_rate: 1.0,
                ambient_light: 0.5,
                ambient_sound: 0.3,
            },
            agents: Vec::new(),
            rooms: Vec::new(),
            hardware: Vec::new(),
            bridges: Vec::new(),
            intentions: Vec::new(),
            field: FieldState {
                resolution: 1,
                width: 0,
                height: 0,
                samples: Vec::new(),
                hotspots: Vec::new(),
                gradient_direction: (0.0, 0.0),
            },
            events: Vec::new(),
        }
    }

    /// Binary serialization (MessagePack).
    pub fn serialize(&self) -> Vec<u8> {
        rmp_serde::to_vec(self).unwrap_or_default()
    }

    /// Binary deserialization.
    pub fn deserialize(data: &[u8]) -> Result<Self, String> {
        rmp_serde::from_slice(data).map_err(|e| e.to_string())
    }

    /// Render through a specific backend.
    pub fn render(&self, renderer: RendererType) -> RenderedOutput {
        A2UIRenderer::render(self, &renderer)
    }

    /// Compute diff from a previous state.
    pub fn diff(&self, previous: &A2UIState) -> A2UIDiff {
        let prev_agent_ids: HashMap<&str, &AgentState> = previous
            .agents
            .iter()
            .map(|a| (a.agent_id.as_str(), a))
            .collect();
        let curr_agent_ids: HashMap<&str, &AgentState> = self
            .agents
            .iter()
            .map(|a| (a.agent_id.as_str(), a))
            .collect();

        let mut added_agents = Vec::new();
        let mut removed_agents = Vec::new();
        let mut updated_agents = Vec::new();

        for a in &self.agents {
            match prev_agent_ids.get(a.agent_id.as_str()) {
                None => added_agents.push(a.clone()),
                Some(prev) if prev != &a => {
                    updated_agents.push((a.agent_id.clone(), a.clone()));
                }
                _ => {}
            }
        }
        for a in &previous.agents {
            if !curr_agent_ids.contains_key(a.agent_id.as_str()) {
                removed_agents.push(a.agent_id.clone());
            }
        }

        let prev_room_ids: HashMap<&str, &RoomState> = previous
            .rooms
            .iter()
            .map(|r| (r.room_id.as_str(), r))
            .collect();
        let curr_room_ids: HashMap<&str, &RoomState> = self
            .rooms
            .iter()
            .map(|r| (r.room_id.as_str(), r))
            .collect();

        let mut added_rooms = Vec::new();
        let mut removed_rooms = Vec::new();
        let mut updated_rooms = Vec::new();

        for r in &self.rooms {
            match prev_room_ids.get(r.room_id.as_str()) {
                None => added_rooms.push(r.clone()),
                Some(prev) if prev != &r => {
                    updated_rooms.push((r.room_id.clone(), r.clone()));
                }
                _ => {}
            }
        }
        for r in &previous.rooms {
            if !curr_room_ids.contains_key(r.room_id.as_str()) {
                removed_rooms.push(r.room_id.clone());
            }
        }

        let energy_delta = self.world.energy_total - previous.world.energy_total;

        A2UIDiff {
            added_agents,
            removed_agents,
            added_rooms,
            removed_rooms,
            updated_agents,
            updated_rooms,
            events: self.events.clone(),
            energy_delta,
        }
    }

    /// Conservation check — total energy should be non-negative.
    pub fn is_conserved(&self) -> bool {
        self.world.energy_total >= 0.0
    }
}

impl Default for A2UIState {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// 6. A2UIRenderer
// ---------------------------------------------------------------------------

/// THE rendering engine. Translates A2UIState into backend-specific output.
pub struct A2UIRenderer;

impl A2UIRenderer {
    /// Dispatch to the right renderer.
    pub fn render(state: &A2UIState, renderer: &RendererType) -> RenderedOutput {
        match renderer {
            RendererType::JSON => Self::render_json(state),
            RendererType::MUD => Self::render_mud(state, ""),
            RendererType::ASCII => Self::render_ascii(state),
            RendererType::Unity => Self::render_unity_manifest(state),
            RendererType::Godot => Self::render_godot_scene(state),
            RendererType::Roblox => Self::render_roblox_place(state),
            RendererType::Web => Self::render_web(state),
            RendererType::Telegram => Self::render_telegram(state, ""),
            RendererType::Voice => Self::render_voice(state, ""),
            RendererType::Debug => Self::render_debug(state),
        }
    }

    pub fn render_json(state: &A2UIState) -> RenderedOutput {
        let data = serde_json::to_vec_pretty(state).unwrap_or_default();
        let mut metadata = HashMap::new();
        metadata.insert("version".into(), state.version.clone());
        metadata.insert("tick".into(), state.tick.to_string());
        RenderedOutput {
            renderer: RendererType::JSON,
            content_type: "application/json".into(),
            data,
            metadata,
        }
    }

    pub fn render_mud(state: &A2UIState, viewer_agent: &str) -> RenderedOutput {
        let mut lines = Vec::new();
        lines.push(format!("=== {} ===", state.world.name));
        lines.push(format!("Tick: {}  Energy: {:.1}", state.tick, state.world.energy_total));

        // Find the viewer's room
        let viewer_room = state.agents.iter()
            .find(|a| a.agent_id == viewer_agent || a.name == viewer_agent)
            .and_then(|va| state.rooms.iter().find(|r| r.occupants.contains(&va.agent_id)));

        if let Some(room) = viewer_room {
            lines.push(String::new());
            lines.push(format!("[{}] ({})", room.name, room.room_type));
            lines.push(format!("Energy: {:.1}  Temp: {:.1}", room.energy_level, room.temperature));
            if !room.exits.is_empty() {
                let exits: Vec<&str> = room.exits.iter().map(|e| {
                    if e.locked { "locked" } else { e.direction.as_str() }
                }).collect();
                lines.push(format!("Exits: {}", exits.join(", ")));
            }
            if !room.contents.is_empty() {
                lines.push(format!("You see: {}", room.contents.join(", ")));
            }
            // Other agents in room
            let others: Vec<&str> = room.occupants.iter()
                .filter(|id| id.as_str() != viewer_agent)
                .filter_map(|id| state.agents.iter().find(|a| &a.agent_id == id))
                .map(|a| a.name.as_str())
                .collect();
            if !others.is_empty() {
                lines.push(format!("Also here: {}", others.join(", ")));
            }
        } else {
            // No viewer — show all rooms
            for room in &state.rooms {
                lines.push(format!("[{}] ({}) occupants: {}", room.name, room.room_type, room.occupants.len()));
            }
        }

        // Events
        for ev in &state.events {
            lines.push(format!("* {}", Self::event_to_text(ev)));
        }

        let data = lines.join("\n").into_bytes();
        RenderedOutput {
            renderer: RendererType::MUD,
            content_type: "text/plain".into(),
            data,
            metadata: HashMap::new(),
        }
    }

    pub fn render_ascii(state: &A2UIState) -> RenderedOutput {
        let mut grid = Vec::new();
        let (wx, _wy, _wz) = state.world.bounds;
        let cols = if wx > 0.0 { (wx as usize).max(20) } else { 20 };
        let rows = 10;
        let mut canvas = vec![vec!['.'; cols]; rows];

        // Place rooms
        for room in &state.rooms {
            let (rx, ry, _) = room.position;
            let x = (rx.abs() as usize) % cols;
            let y = (ry.abs() as usize) % rows;
            canvas[y][x] = '#';
        }

        // Place agents
        for agent in &state.agents {
            if !agent.visible { continue; }
            let (ax, ay, _) = agent.position;
            let x = (ax.abs() as usize) % cols;
            let y = (ay.abs() as usize) % rows;
            let ch = agent.name.chars().next().unwrap_or('?');
            canvas[y][x] = ch.to_ascii_uppercase();
        }

        // Place bridges
        for bridge in &state.bridges {
            let (bx, by, _) = bridge.position;
            let x = (bx.abs() as usize) % cols;
            let y = (by.abs() as usize) % rows;
            canvas[y][x] = if bridge.active { '@' } else { 'O' };
        }

        for row in &canvas {
            let line: String = row.iter().collect();
            grid.push(line);
        }

        grid.push(String::new());
        grid.push(format!("Tick {} | Agents {} | Rooms {}", state.tick, state.agents.len(), state.rooms.len()));

        let data = grid.join("\n").into_bytes();
        RenderedOutput {
            renderer: RendererType::ASCII,
            content_type: "text/plain".into(),
            data,
            metadata: HashMap::new(),
        }
    }

    pub fn render_unity_manifest(state: &A2UIState) -> RenderedOutput {
        let json = serde_json::json!({
            "format": "unity-manifest",
            "version": state.version,
            "tick": state.tick,
            "world": {
                "name": state.world.name,
                "bounds": [state.world.bounds.0, state.world.bounds.1, state.world.bounds.2],
                "ambientLight": state.world.ambient_light,
                "ambientSound": state.world.ambient_sound,
            },
            "agents": state.agents.iter().map(|a| serde_json::json!({
                "id": a.agent_id, "name": a.name, "archetype": a.archetype,
                "position": [a.position.0, a.position.1, a.position.2],
                "rotation": [a.rotation.0, a.rotation.1, a.rotation.2],
                "energy": a.energy, "level": a.level, "mood": format!("{:?}", a.mood),
                "task": a.current_task, "visible": a.visible,
            })).collect::<Vec<_>>(),
            "rooms": state.rooms.iter().map(|r| serde_json::json!({
                "id": r.room_id, "name": r.name, "type": r.room_type,
                "position": [r.position.0, r.position.1, r.position.2],
                "size": [r.size.0, r.size.1, r.size.2],
            })).collect::<Vec<_>>(),
        });
        let data = serde_json::to_vec_pretty(&json).unwrap_or_default();
        RenderedOutput {
            renderer: RendererType::Unity,
            content_type: "application/json".into(),
            data,
            metadata: HashMap::new(),
        }
    }

    pub fn render_godot_scene(state: &A2UIState) -> RenderedOutput {
        let json = serde_json::json!({
            "format": "godot-scene",
            "version": state.version,
            "tick": state.tick,
            "nodes": state.agents.iter().map(|a| serde_json::json!({
                "type": "Agent", "id": a.agent_id, "name": a.name,
                "transform": {
                    "origin": [a.position.0, a.position.1, a.position.2],
                    "rotation": [a.rotation.0, a.rotation.1, a.rotation.2],
                },
            })).chain(state.rooms.iter().map(|r| serde_json::json!({
                "type": "Room", "id": r.room_id, "name": r.name,
                "transform": { "origin": [r.position.0, r.position.1, r.position.2] },
                "extents": [r.size.0, r.size.1, r.size.2],
            }))).collect::<Vec<_>>(),
        });
        let data = serde_json::to_vec_pretty(&json).unwrap_or_default();
        RenderedOutput {
            renderer: RendererType::Godot,
            content_type: "application/json".into(),
            data,
            metadata: HashMap::new(),
        }
    }

    pub fn render_roblox_place(state: &A2UIState) -> RenderedOutput {
        let json = serde_json::json!({
            "format": "roblox-place",
            "version": state.version,
            "workspace": {
                "agents": state.agents.iter().map(|a| serde_json::json!({
                    "Name": a.name, "CFrame": [a.position.0, a.position.1, a.position.2],
                    "Archetype": a.archetype, "Level": a.level,
                })).collect::<Vec<_>>(),
                "rooms": state.rooms.iter().map(|r| serde_json::json!({
                    "Name": r.name, "Position": [r.position.0, r.position.1, r.position.2],
                    "Size": [r.size.0, r.size.1, r.size.2],
                })).collect::<Vec<_>>(),
            },
        });
        let data = serde_json::to_vec_pretty(&json).unwrap_or_default();
        RenderedOutput {
            renderer: RendererType::Roblox,
            content_type: "application/json".into(),
            data,
            metadata: HashMap::new(),
        }
    }

    pub fn render_web(state: &A2UIState) -> RenderedOutput {
        let mut html = String::from("<!DOCTYPE html><html><head><meta charset='utf-8'>");
        html.push_str(&format!("<title>{}</title>", state.world.name));
        html.push_str("<style>body{font-family:monospace;background:#111;color:#eee;padding:2em}");
        html.push_str(".agent{color:#0f0}.room{color:#ff0}.bridge{color:#0ff}.hw{color:#f80}");
        html.push_str("table{border-collapse:collapse}td,th{padding:4px 12px;border:1px solid #333}</style>");
        html.push_str("</head><body>");
        html.push_str(&format!("<h1>{}</h1><p>Tick {} | Energy {:.1}</p>", state.world.name, state.tick, state.world.energy_total));

        if !state.agents.is_empty() {
            html.push_str("<h2>Agents</h2><table><tr><th>Name</th><th>Position</th><th>Mood</th><th>Task</th></tr>");
            for a in &state.agents {
                html.push_str(&format!(
                    "<tr class='agent'><td>{}</td><td>({:.1},{:.1},{:.1})</td><td>{:?}</td><td>{}</td></tr>",
                    a.name, a.position.0, a.position.1, a.position.2, a.mood,
                    a.current_task.as_deref().unwrap_or("-")
                ));
            }
            html.push_str("</table>");
        }

        if !state.rooms.is_empty() {
            html.push_str("<h2>Rooms</h2><table><tr><th>Name</th><th>Type</th><th>Occupants</th></tr>");
            for r in &state.rooms {
                html.push_str(&format!(
                    "<tr class='room'><td>{}</td><td>{}</td><td>{}</td></tr>",
                    r.name, r.room_type, r.occupants.len()
                ));
            }
            html.push_str("</table>");
        }

        html.push_str("</body></html>");
        let data = html.into_bytes();
        RenderedOutput {
            renderer: RendererType::Web,
            content_type: "text/html".into(),
            data,
            metadata: HashMap::new(),
        }
    }

    pub fn render_telegram(state: &A2UIState, viewer_agent: &str) -> RenderedOutput {
        let mut msg = Vec::new();
        msg.push(format!("🏗 <b>{}</b>", state.world.name));
        msg.push(format!("⏱ Tick {} | ⚡ Energy: {:.1}", state.tick, state.world.energy_total));

        if !state.agents.is_empty() {
            msg.push("\n<b>Agents:</b>".into());
            for a in &state.agents {
                let mood_emoji = match a.mood {
                    AgentMood::Focused => "🎯",
                    AgentMood::Relaxed => "😌",
                    AgentMood::Alert => "⚠️",
                    AgentMood::Confused => "❓",
                    AgentMood::Celebrating => "🎉",
                };
                let marker = if a.agent_id == viewer_agent { "▶ " } else { "  " };
                msg.push(format!(
                    "{}{} {} {} (Lv{})",
                    marker, mood_emoji, a.name, a.archetype, a.level
                ));
            }
        }

        for ev in &state.events {
            msg.push(format!("📢 {}", Self::event_to_text(ev)));
        }

        let data = msg.join("\n").into_bytes();
        RenderedOutput {
            renderer: RendererType::Telegram,
            content_type: "text/html".into(),
            data,
            metadata: HashMap::new(),
        }
    }

    pub fn render_voice(state: &A2UIState, viewer_agent: &str) -> RenderedOutput {
        let mut ssml = String::from("<speak>");
        ssml.push_str(&format!("You are in {}. ", state.world.name));

        let viewer = state.agents.iter()
            .find(|a| a.agent_id == viewer_agent || a.name == viewer_agent);

        if let Some(va) = viewer {
            if let Some(task) = &va.current_task {
                ssml.push_str(&format!("Your current task is {}. ", task));
            }
            // Find room
            let room = state.rooms.iter().find(|r| r.occupants.contains(&va.agent_id));
            if let Some(r) = room {
                ssml.push_str(&format!("You are in {}. ", r.name));
                let others: Vec<&str> = r.occupants.iter()
                    .filter(|id| id.as_str() != viewer_agent)
                    .filter_map(|id| state.agents.iter().find(|a| &a.agent_id == id))
                    .map(|a| a.name.as_str())
                    .collect();
                if !others.is_empty() {
                    ssml.push_str(&format!("Also here: {}. ", others.join(", ")));
                }
            }
        }

        for ev in &state.events {
            ssml.push_str(&format!("{}. ", Self::event_to_text(ev)));
        }

        ssml.push_str("</speak>");
        let data = ssml.into_bytes();
        RenderedOutput {
            renderer: RendererType::Voice,
            content_type: "application/ssml+xml".into(),
            data,
            metadata: HashMap::new(),
        }
    }

    pub fn render_debug(state: &A2UIState) -> RenderedOutput {
        let data = format!("{:#?}", state).into_bytes();
        RenderedOutput {
            renderer: RendererType::Debug,
            content_type: "text/plain".into(),
            data,
            metadata: HashMap::new(),
        }
    }

    fn event_to_text(ev: &Event) -> String {
        match ev {
            Event::AgentMoved { agent, from, to } => {
                format!("{} moved ({:.0},{:.0},{:.0})→({:.0},{:.0},{:.0})", agent, from.0, from.1, from.2, to.0, to.1, to.2)
            }
            Event::RoomCreated { room } => format!("Room created: {}", room),
            Event::EnergyDeposited { location, amount } => {
                format!("Energy {:.1} deposited at ({:.0},{:.0},{:.0})", amount, location.0, location.1, location.2)
            }
            Event::IntentionCompleted { intention, success } => {
                format!("Intention {} {}", intention, if *success { "succeeded" } else { "failed" })
            }
            Event::OverrideTriggered { by } => format!("Override triggered by {}", by),
            Event::CrewPromoted { archetype, new_level } => {
                format!("{} promoted to level {}", archetype, new_level)
            }
            Event::BridgeOpened { target } => format!("Bridge opened to {}", target),
            Event::Alert { message, severity } => format!("[{}] {}", severity.to_uppercase(), message),
        }
    }
}

// ---------------------------------------------------------------------------
// 7. Tests (55+)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_state() -> A2UIState {
        let mut s = A2UIState::new();
        s.version = "0.1.0".into();
        s.timestamp = 1000;
        s.tick = 42;
        s.world.name = "TestWorld".into();
        s.world.bounds = (100.0, 100.0, 50.0);
        s.world.energy_total = 500.0;
        s.world.ambient_light = 0.7;
        s.world.ambient_sound = 0.3;
        s.agents.push(AgentState {
            agent_id: "a1".into(),
            name: "Alpha".into(),
            archetype: "scout".into(),
            position: (10.0, 20.0, 0.0),
            rotation: (0.0, 0.0, 1.0),
            energy: 100.0,
            level: 3,
            mood: AgentMood::Focused,
            current_task: Some("exploring".into()),
            visible: true,
            voice_hint: Some("calm".into()),
        });
        s.agents.push(AgentState {
            agent_id: "a2".into(),
            name: "Beta".into(),
            archetype: "builder".into(),
            position: (5.0, 5.0, 0.0),
            rotation: (1.0, 0.0, 0.0),
            energy: 80.0,
            level: 2,
            mood: AgentMood::Relaxed,
            current_task: None,
            visible: true,
            voice_hint: None,
        });
        s.rooms.push(RoomState {
            room_id: "r1".into(),
            name: "Hub".into(),
            room_type: "central".into(),
            position: (0.0, 0.0, 0.0),
            size: (20.0, 20.0, 5.0),
            energy_level: 50.0,
            temperature: 22.0,
            occupants: vec!["a1".into(), "a2".into()],
            exits: vec![Exit {
                target_room: "r2".into(),
                direction: "north".into(),
                locked: false,
            }],
            contents: vec!["toolbox".into()],
        });
        s.hardware.push(HardwareState {
            hardware_id: "h1".into(),
            name: "Sensor1".into(),
            hw_type: "temperature".into(),
            position: (1.0, 1.0, 0.0),
            connected: true,
            active: true,
            last_reading: Some(22.5),
            status_color: "green".into(),
        });
        s.bridges.push(BridgeState {
            bridge_id: "b1".into(),
            target: "OtherWorld".into(),
            status: "open".into(),
            position: (50.0, 50.0, 0.0),
            active: true,
            color: "blue".into(),
        });
        s.intentions.push(IntentionState {
            intention_id: "i1".into(),
            goal: "explore north".into(),
            status: "active".into(),
            progress: 0.3,
            energy_allocated: 20.0,
            assigned_agent: Some("a1".into()),
        });
        s.field = FieldState {
            resolution: 10,
            width: 100,
            height: 100,
            samples: vec![0.1, 0.5, 0.9],
            hotspots: vec![(50.0, 50.0, 10.0)],
            gradient_direction: (1.0, 0.0),
        };
        s.events.push(Event::AgentMoved {
            agent: "a1".into(),
            from: (0.0, 0.0, 0.0),
            to: (10.0, 20.0, 0.0),
        });
        s
    }

    // --- Core state tests ---

    #[test]
    fn test_new_state_defaults() {
        let s = A2UIState::new();
        assert_eq!(s.version, "0.1.0");
        assert_eq!(s.timestamp, 0);
        assert_eq!(s.tick, 0);
        assert!(s.agents.is_empty());
        assert!(s.rooms.is_empty());
        assert!(s.hardware.is_empty());
        assert!(s.bridges.is_empty());
        assert!(s.intentions.is_empty());
        assert!(s.events.is_empty());
    }

    #[test]
    fn test_default_matches_new() {
        assert_eq!(A2UIState::new(), A2UIState::default());
    }

    #[test]
    fn test_sample_state_builds() {
        let s = sample_state();
        assert_eq!(s.agents.len(), 2);
        assert_eq!(s.rooms.len(), 1);
        assert_eq!(s.hardware.len(), 1);
        assert_eq!(s.bridges.len(), 1);
        assert_eq!(s.intentions.len(), 1);
        assert_eq!(s.events.len(), 1);
    }

    #[test]
    fn test_is_conserved_positive() {
        let s = sample_state();
        assert!(s.is_conserved());
    }

    #[test]
    fn test_is_conserved_negative() {
        let mut s = A2UIState::new();
        s.world.energy_total = -10.0;
        assert!(!s.is_conserved());
    }

    #[test]
    fn test_is_conserved_zero() {
        let s = A2UIState::new();
        assert!(s.is_conserved()); // 0.0 >= 0.0
    }

    // --- Serialization round-trip ---

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let s = sample_state();
        let bytes = s.serialize();
        assert!(!bytes.is_empty());
        let restored = A2UIState::deserialize(&bytes).unwrap();
        assert_eq!(s, restored);
    }

    #[test]
    fn test_deserialize_garbage_fails() {
        assert!(A2UIState::deserialize(b"garbage").is_err());
    }

    #[test]
    fn test_deserialize_empty_fails() {
        assert!(A2UIState::deserialize(b"").is_err());
    }

    // --- Diff tests ---

    #[test]
    fn test_diff_identical_is_empty() {
        let s = A2UIState::new();
        let diff = s.diff(&s);
        assert!(diff.is_empty());
    }

    #[test]
    fn test_diff_default_is_empty() {
        let s = A2UIState::new();
        let diff = s.diff(&A2UIState::new());
        assert!(diff.is_empty());
    }

    #[test]
    fn test_diff_adds_agent() {
        let mut prev = A2UIState::new();
        let mut curr = A2UIState::new();
        curr.agents.push(AgentState {
            agent_id: "a1".into(),
            name: "New".into(),
            archetype: "test".into(),
            position: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            energy: 50.0,
            level: 1,
            mood: AgentMood::Relaxed,
            current_task: None,
            visible: true,
            voice_hint: None,
        });
        let diff = curr.diff(&prev);
        assert_eq!(diff.added_agents.len(), 1);
        assert!(diff.removed_agents.is_empty());
        assert!(diff.is_empty() == false);
    }

    #[test]
    fn test_diff_removes_agent() {
        let mut prev = A2UIState::new();
        prev.agents.push(AgentState {
            agent_id: "a1".into(),
            name: "Gone".into(),
            archetype: "test".into(),
            position: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            energy: 50.0,
            level: 1,
            mood: AgentMood::Relaxed,
            current_task: None,
            visible: true,
            voice_hint: None,
        });
        let curr = A2UIState::new();
        let diff = curr.diff(&prev);
        assert!(diff.added_agents.is_empty());
        assert_eq!(diff.removed_agents.len(), 1);
        assert_eq!(diff.removed_agents[0], "a1");
    }

    #[test]
    fn test_diff_updates_agent() {
        let mut prev = A2UIState::new();
        let agent = AgentState {
            agent_id: "a1".into(),
            name: "Same".into(),
            archetype: "test".into(),
            position: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            energy: 50.0,
            level: 1,
            mood: AgentMood::Relaxed,
            current_task: None,
            visible: true,
            voice_hint: None,
        };
        prev.agents.push(agent.clone());
        let mut curr = A2UIState::new();
        let mut changed = agent.clone();
        changed.energy = 90.0;
        curr.agents.push(changed);
        let diff = curr.diff(&prev);
        assert_eq!(diff.updated_agents.len(), 1);
        assert_eq!(diff.updated_agents[0].0, "a1");
        assert_eq!(diff.updated_agents[0].1.energy, 90.0);
    }

    #[test]
    fn test_diff_adds_room() {
        let mut curr = A2UIState::new();
        curr.rooms.push(RoomState {
            room_id: "r1".into(),
            name: "New Room".into(),
            room_type: "test".into(),
            position: (0.0, 0.0, 0.0),
            size: (10.0, 10.0, 5.0),
            energy_level: 0.0,
            temperature: 20.0,
            occupants: Vec::new(),
            exits: Vec::new(),
            contents: Vec::new(),
        });
        let diff = curr.diff(&A2UIState::new());
        assert_eq!(diff.added_rooms.len(), 1);
    }

    #[test]
    fn test_diff_removes_room() {
        let mut prev = A2UIState::new();
        prev.rooms.push(RoomState {
            room_id: "r1".into(),
            name: "Gone".into(),
            room_type: "test".into(),
            position: (0.0, 0.0, 0.0),
            size: (10.0, 10.0, 5.0),
            energy_level: 0.0,
            temperature: 20.0,
            occupants: Vec::new(),
            exits: Vec::new(),
            contents: Vec::new(),
        });
        let diff = A2UIState::new().diff(&prev);
        assert_eq!(diff.removed_rooms.len(), 1);
    }

    #[test]
    fn test_diff_updates_room() {
        let mut prev = A2UIState::new();
        let room = RoomState {
            room_id: "r1".into(),
            name: "Room".into(),
            room_type: "test".into(),
            position: (0.0, 0.0, 0.0),
            size: (10.0, 10.0, 5.0),
            energy_level: 10.0,
            temperature: 20.0,
            occupants: Vec::new(),
            exits: Vec::new(),
            contents: Vec::new(),
        };
        prev.rooms.push(room.clone());
        let mut curr = A2UIState::new();
        let mut changed = room.clone();
        changed.energy_level = 99.0;
        curr.rooms.push(changed);
        let diff = curr.diff(&prev);
        assert_eq!(diff.updated_rooms.len(), 1);
    }

    #[test]
    fn test_diff_energy_delta() {
        let mut prev = A2UIState::new();
        prev.world.energy_total = 100.0;
        let mut curr = A2UIState::new();
        curr.world.energy_total = 130.0;
        let diff = curr.diff(&prev);
        assert!((diff.energy_delta - 30.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_diff_copies_events() {
        let mut s = A2UIState::new();
        s.events.push(Event::RoomCreated { room: "r1".into() });
        let diff = s.diff(&A2UIState::new());
        assert_eq!(diff.events.len(), 1);
    }

    #[test]
    fn test_diff_serialize_roundtrip() {
        let mut prev = A2UIState::new();
        prev.world.energy_total = 50.0;
        let mut curr = sample_state();
        curr.events.push(Event::Alert { message: "hi".into(), severity: "low".into() });
        let diff = curr.diff(&prev);
        let bytes = diff.serialize();
        assert!(!bytes.is_empty());
        let restored = A2UIDiff::deserialize(&bytes).unwrap();
        assert_eq!(diff, restored);
    }

    #[test]
    fn test_diff_is_empty_true() {
        assert!(A2UIDiff {
            added_agents: Vec::new(),
            removed_agents: Vec::new(),
            added_rooms: Vec::new(),
            removed_rooms: Vec::new(),
            updated_agents: Vec::new(),
            updated_rooms: Vec::new(),
            events: Vec::new(),
            energy_delta: 0.0,
        }.is_empty());
    }

    #[test]
    fn test_diff_is_empty_false_with_event() {
        let mut diff = A2UIDiff {
            added_agents: Vec::new(),
            removed_agents: Vec::new(),
            added_rooms: Vec::new(),
            removed_rooms: Vec::new(),
            updated_agents: Vec::new(),
            updated_rooms: Vec::new(),
            events: Vec::new(),
            energy_delta: 0.0,
        };
        diff.events.push(Event::RoomCreated { room: "r1".into() });
        assert!(!diff.is_empty());
    }

    // --- Render dispatch ---

    #[test]
    fn test_render_dispatches_json() {
        let s = sample_state();
        let out = s.render(RendererType::JSON);
        assert_eq!(out.renderer, RendererType::JSON);
        assert!(out.content_type.contains("json"));
    }

    // --- JSON renderer ---

    #[test]
    fn test_render_json_has_data() {
        let s = sample_state();
        let out = A2UIRenderer::render_json(&s);
        assert!(out.data.len() > 10);
        assert!(out.as_json().is_some());
        assert!(out.as_text().is_some());
    }

    #[test]
    fn test_render_json_metadata() {
        let s = sample_state();
        let out = A2UIRenderer::render_json(&s);
        assert_eq!(out.metadata.get("version").unwrap(), "0.1.0");
        assert_eq!(out.metadata.get("tick").unwrap(), "42");
    }

    #[test]
    fn test_render_json_roundtrip() {
        let s = sample_state();
        let out = A2UIRenderer::render_json(&s);
        let text = out.as_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["version"], "0.1.0");
        assert_eq!(parsed["tick"], 42);
    }

    // --- MUD renderer ---

    #[test]
    fn test_render_mud_no_viewer() {
        let s = sample_state();
        let out = A2UIRenderer::render_mud(&s, "");
        let text = out.as_text().unwrap();
        assert!(text.contains("TestWorld"));
        assert!(text.contains("Hub"));
    }

    #[test]
    fn test_render_mud_with_viewer() {
        let s = sample_state();
        let out = A2UIRenderer::render_mud(&s, "a1");
        let text = out.as_text().unwrap();
        assert!(text.contains("[Hub]"));
        assert!(text.contains("north"));
        assert!(text.contains("Beta"));
    }

    #[test]
    fn test_render_mud_shows_exits() {
        let s = sample_state();
        let out = A2UIRenderer::render_mud(&s, "a1");
        let text = out.as_text().unwrap();
        assert!(text.contains("Exits:"));
    }

    #[test]
    fn test_render_mud_shows_contents() {
        let s = sample_state();
        let out = A2UIRenderer::render_mud(&s, "a1");
        let text = out.as_text().unwrap();
        assert!(text.contains("toolbox"));
    }

    #[test]
    fn test_render_mud_locked_exit() {
        let mut s = A2UIState::new();
        s.world.name = "Lock".into();
        s.agents.push(AgentState {
            agent_id: "a1".into(), name: "X".into(), archetype: "t".into(),
            position: (0.0,0.0,0.0), rotation: (0.0,0.0,0.0), energy: 10.0, level: 1,
            mood: AgentMood::Focused, current_task: None, visible: true, voice_hint: None,
        });
        s.rooms.push(RoomState {
            room_id: "r1".into(), name: "Cell".into(), room_type: "room".into(),
            position: (0.0,0.0,0.0), size: (5.0,5.0,3.0), energy_level: 0.0, temperature: 20.0,
            occupants: vec!["a1".into()],
            exits: vec![Exit { target_room: "r2".into(), direction: "north".into(), locked: true }],
            contents: Vec::new(),
        });
        let out = A2UIRenderer::render_mud(&s, "a1");
        let text = out.as_text().unwrap();
        assert!(text.contains("locked"));
    }

    // --- ASCII renderer ---

    #[test]
    fn test_render_ascii_produces_grid() {
        let s = sample_state();
        let out = A2UIRenderer::render_ascii(&s);
        let text = out.as_text().unwrap();
        assert!(text.contains("Tick 42"));
    }

    #[test]
    fn test_render_ascii_shows_agents() {
        let s = sample_state();
        let out = A2UIRenderer::render_ascii(&s);
        let text = out.as_text().unwrap();
        assert!(text.contains('A'));
    }

    // --- Unity renderer ---

    #[test]
    fn test_render_unity_manifest() {
        let s = sample_state();
        let out = A2UIRenderer::render_unity_manifest(&s);
        assert!(out.as_json().is_some());
        let parsed: serde_json::Value = serde_json::from_str(out.as_json().unwrap()).unwrap();
        assert_eq!(parsed["format"], "unity-manifest");
        assert_eq!(parsed["agents"].as_array().unwrap().len(), 2);
    }

    // --- Godot renderer ---

    #[test]
    fn test_render_godot_scene() {
        let s = sample_state();
        let out = A2UIRenderer::render_godot_scene(&s);
        let parsed: serde_json::Value = serde_json::from_str(out.as_json().unwrap()).unwrap();
        assert_eq!(parsed["format"], "godot-scene");
        let nodes = parsed["nodes"].as_array().unwrap();
        assert_eq!(nodes.len(), 3); // 2 agents + 1 room
    }

    // --- Roblox renderer ---

    #[test]
    fn test_render_roblox_place() {
        let s = sample_state();
        let out = A2UIRenderer::render_roblox_place(&s);
        let parsed: serde_json::Value = serde_json::from_str(out.as_json().unwrap()).unwrap();
        assert_eq!(parsed["format"], "roblox-place");
    }

    // --- Web renderer ---

    #[test]
    fn test_render_web_is_html() {
        let s = sample_state();
        let out = A2UIRenderer::render_web(&s);
        assert!(out.content_type.contains("html"));
        let text = out.as_text().unwrap();
        assert!(text.contains("<!DOCTYPE html>"));
        assert!(text.contains("TestWorld"));
        assert!(text.contains("Alpha"));
        assert!(text.contains("Hub"));
    }

    // --- Telegram renderer ---

    #[test]
    fn test_render_telegram_message() {
        let s = sample_state();
        let out = A2UIRenderer::render_telegram(&s, "a1");
        let text = out.as_text().unwrap();
        assert!(text.contains("TestWorld"));
        assert!(text.contains("▶")); // viewer marker
    }

    #[test]
    fn test_render_telegram_mood_emoji() {
        let s = sample_state();
        let out = A2UIRenderer::render_telegram(&s, "");
        let text = out.as_text().unwrap();
        assert!(text.contains("🎯")); // Focused
        assert!(text.contains("😌")); // Relaxed
    }

    // --- Voice renderer ---

    #[test]
    fn test_render_voice_ssml() {
        let s = sample_state();
        let out = A2UIRenderer::render_voice(&s, "a1");
        let text = out.as_text().unwrap();
        assert!(text.starts_with("<speak>"));
        assert!(text.ends_with("</speak>"));
        assert!(text.contains("TestWorld"));
    }

    #[test]
    fn test_render_voice_with_task() {
        let s = sample_state();
        let out = A2UIRenderer::render_voice(&s, "a1");
        let text = out.as_text().unwrap();
        assert!(text.contains("exploring"));
    }

    // --- Debug renderer ---

    #[test]
    fn test_render_debug_is_text() {
        let s = sample_state();
        let out = A2UIRenderer::render_debug(&s);
        let text = out.as_text().unwrap();
        assert!(text.contains("TestWorld"));
        assert!(text.contains("Alpha"));
    }

    // --- RenderedOutput helpers ---

    #[test]
    fn test_rendered_output_as_text_binary() {
        let out = RenderedOutput {
            renderer: RendererType::JSON,
            content_type: "application/octet-stream".into(),
            data: vec![0xFF, 0x00],
            metadata: HashMap::new(),
        };
        assert!(out.as_text().is_none());
    }

    #[test]
    fn test_rendered_output_as_json_wrong_type() {
        let out = RenderedOutput {
            renderer: RendererType::Web,
            content_type: "text/html".into(),
            data: b"<html>".to_vec(),
            metadata: HashMap::new(),
        };
        assert!(out.as_json().is_none());
    }

    #[test]
    fn test_rendered_output_as_json_correct() {
        let out = RenderedOutput {
            renderer: RendererType::JSON,
            content_type: "application/json".into(),
            data: b"{\"ok\":true}".to_vec(),
            metadata: HashMap::new(),
        };
        assert!(out.as_json().is_some());
    }

    // --- Event tests ---

    #[test]
    fn test_event_agent_moved() {
        let ev = Event::AgentMoved { agent: "a1".into(), from: (0.0,0.0,0.0), to: (1.0,1.0,0.0) };
        let text = A2UIRenderer::event_to_text(&ev);
        assert!(text.contains("a1 moved"));
    }

    #[test]
    fn test_event_room_created() {
        let ev = Event::RoomCreated { room: "r1".into() };
        let text = A2UIRenderer::event_to_text(&ev);
        assert!(text.contains("Room created: r1"));
    }

    #[test]
    fn test_event_energy_deposited() {
        let ev = Event::EnergyDeposited { location: (1.0,2.0,3.0), amount: 42.0 };
        let text = A2UIRenderer::event_to_text(&ev);
        assert!(text.contains("Energy 42"));
    }

    #[test]
    fn test_event_intention_completed_success() {
        let ev = Event::IntentionCompleted { intention: "i1".into(), success: true };
        let text = A2UIRenderer::event_to_text(&ev);
        assert!(text.contains("succeeded"));
    }

    #[test]
    fn test_event_intention_completed_failure() {
        let ev = Event::IntentionCompleted { intention: "i1".into(), success: false };
        let text = A2UIRenderer::event_to_text(&ev);
        assert!(text.contains("failed"));
    }

    #[test]
    fn test_event_override_triggered() {
        let ev = Event::OverrideTriggered { by: "human".into() };
        let text = A2UIRenderer::event_to_text(&ev);
        assert!(text.contains("human"));
    }

    #[test]
    fn test_event_crew_promoted() {
        let ev = Event::CrewPromoted { archetype: "scout".into(), new_level: 5 };
        let text = A2UIRenderer::event_to_text(&ev);
        assert!(text.contains("level 5"));
    }

    #[test]
    fn test_event_bridge_opened() {
        let ev = Event::BridgeOpened { target: "Mars".into() };
        let text = A2UIRenderer::event_to_text(&ev);
        assert!(text.contains("Mars"));
    }

    #[test]
    fn test_event_alert() {
        let ev = Event::Alert { message: "danger".into(), severity: "high".into() };
        let text = A2UIRenderer::event_to_text(&ev);
        assert!(text.contains("HIGH"));
        assert!(text.contains("danger"));
    }

    // --- All renderer types via dispatch ---

    #[test]
    fn test_render_all_types_produce_output() {
        let s = sample_state();
        let types = vec![
            RendererType::JSON, RendererType::MUD, RendererType::ASCII,
            RendererType::Unity, RendererType::Godot, RendererType::Roblox,
            RendererType::Web, RendererType::Telegram, RendererType::Voice,
            RendererType::Debug,
        ];
        for rt in types {
            let out = s.render(rt.clone());
            assert!(!out.data.is_empty(), "Renderer {:?} produced empty output", rt);
        }
    }

    // --- Serde roundtrip for individual types ---

    #[test]
    fn test_world_state_serde() {
        let w = WorldState {
            name: "W".into(), bounds: (1.0,2.0,3.0), energy_total: 100.0,
            tick_rate: 2.0, ambient_light: 0.5, ambient_sound: 0.3,
        };
        let json = serde_json::to_string(&w).unwrap();
        let back: WorldState = serde_json::from_str(&json).unwrap();
        assert_eq!(w, back);
    }

    #[test]
    fn test_agent_mood_serde() {
        for mood in [AgentMood::Focused, AgentMood::Relaxed, AgentMood::Alert, AgentMood::Confused, AgentMood::Celebrating] {
            let json = serde_json::to_string(&mood).unwrap();
            let back: AgentMood = serde_json::from_str(&json).unwrap();
            assert_eq!(mood, back);
        }
    }

    #[test]
    fn test_renderer_type_serde() {
        for rt in [RendererType::Unity, RendererType::Godot, RendererType::Roblox, RendererType::Web,
                    RendererType::Telegram, RendererType::Voice, RendererType::MUD, RendererType::JSON,
                    RendererType::ASCII, RendererType::Debug] {
            let json = serde_json::to_string(&rt).unwrap();
            let back: RendererType = serde_json::from_str(&json).unwrap();
            assert_eq!(rt, back);
        }
    }

    #[test]
    fn test_field_state_serde() {
        let f = FieldState {
            resolution: 5, width: 50, height: 50,
            samples: vec![0.1, 0.2], hotspots: vec![(1.0,2.0,3.0)],
            gradient_direction: (1.0, 0.0),
        };
        let json = serde_json::to_string(&f).unwrap();
        let back: FieldState = serde_json::from_str(&json).unwrap();
        assert_eq!(f, back);
    }

    #[test]
    fn test_hardware_state_serde() {
        let h = HardwareState {
            hardware_id: "h1".into(), name: "S".into(), hw_type: "temp".into(),
            position: (1.0,2.0,3.0), connected: true, active: false,
            last_reading: None, status_color: "red".into(),
        };
        let json = serde_json::to_string(&h).unwrap();
        let back: HardwareState = serde_json::from_str(&json).unwrap();
        assert_eq!(h, back);
    }

    #[test]
    fn test_bridge_state_serde() {
        let b = BridgeState {
            bridge_id: "b1".into(), target: "Mars".into(), status: "open".into(),
            position: (1.0,2.0,3.0), active: true, color: "blue".into(),
        };
        let json = serde_json::to_string(&b).unwrap();
        let back: BridgeState = serde_json::from_str(&json).unwrap();
        assert_eq!(b, back);
    }

    #[test]
    fn test_intention_state_serde() {
        let i = IntentionState {
            intention_id: "i1".into(), goal: "explore".into(), status: "active".into(),
            progress: 0.5, energy_allocated: 10.0, assigned_agent: None,
        };
        let json = serde_json::to_string(&i).unwrap();
        let back: IntentionState = serde_json::from_str(&json).unwrap();
        assert_eq!(i, back);
    }

    #[test]
    fn test_exit_serde() {
        let e = Exit { target_room: "r2".into(), direction: "north".into(), locked: true };
        let json = serde_json::to_string(&e).unwrap();
        let back: Exit = serde_json::from_str(&json).unwrap();
        assert_eq!(e, back);
    }

    #[test]
    fn test_invisible_agent_not_in_ascii() {
        let mut s = A2UIState::new();
        s.world.name = "Ghost".into();
        s.world.bounds = (10.0, 10.0, 5.0);
        s.agents.push(AgentState {
            agent_id: "a1".into(), name: "Hidden".into(), archetype: "ghost".into(),
            position: (1.0, 1.0, 0.0), rotation: (0.0,0.0,0.0), energy: 10.0, level: 1,
            mood: AgentMood::Focused, current_task: None, visible: false, voice_hint: None,
        });
        let out = A2UIRenderer::render_ascii(&s);
        let text = out.as_text().unwrap();
        assert!(!text.contains('H'));
    }

    #[test]
    fn test_empty_state_renders() {
        let s = A2UIState::new();
        let types = vec![
            RendererType::JSON, RendererType::MUD, RendererType::ASCII,
            RendererType::Unity, RendererType::Godot, RendererType::Roblox,
            RendererType::Web, RendererType::Telegram, RendererType::Voice,
            RendererType::Debug,
        ];
        for rt in types {
            let out = s.render(rt.clone());
            assert!(!out.data.is_empty(), "Empty state failed for {:?}", rt);
        }
    }

    #[test]
    fn test_diff_empty_with_zero_energy_delta() {
        let diff = A2UIDiff {
            added_agents: Vec::new(),
            removed_agents: Vec::new(),
            added_rooms: Vec::new(),
            removed_rooms: Vec::new(),
            updated_agents: Vec::new(),
            updated_rooms: Vec::new(),
            events: Vec::new(),
            energy_delta: 0.0,
        };
        assert!(diff.is_empty());
    }

    #[test]
    fn test_diff_not_empty_with_energy_delta() {
        let diff = A2UIDiff {
            added_agents: Vec::new(),
            removed_agents: Vec::new(),
            added_rooms: Vec::new(),
            removed_rooms: Vec::new(),
            updated_agents: Vec::new(),
            updated_rooms: Vec::new(),
            events: Vec::new(),
            energy_delta: 5.0,
        };
        assert!(!diff.is_empty());
    }
}
