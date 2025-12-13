//! Mesh assembly simulation with event recording.

use std::collections::HashMap;

use citadel_topology::{HexCoord, SpiralIndex, spiral_to_coord, Neighbors};
use citadel_consensus::validation_threshold;

use crate::events::{MeshEvent, NodeId, NodeState, ConnectionState, MeshSnapshot};

/// Configuration for the simulation.
#[derive(Debug, Clone)]
pub struct SimulationConfig {
    /// Seed for deterministic simulation
    pub seed: u64,
    /// Whether to simulate network delays
    pub simulate_delays: bool,
    /// Probability of Byzantine behavior (0.0 - 1.0)
    pub byzantine_rate: f64,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            simulate_delays: false,
            byzantine_rate: 0.0,
        }
    }
}

/// Simulates mesh assembly and records events.
pub struct Simulation {
    config: SimulationConfig,
    events: Vec<MeshEvent>,
    nodes: HashMap<NodeId, NodeState>,
    slot_to_node: HashMap<SpiralIndex, NodeId>,
    next_node_id: u64,
    current_frame: u64,
    frontier: SpiralIndex,
}

impl Simulation {
    /// Create a new simulation with the given configuration.
    pub fn new(config: SimulationConfig) -> Self {
        Self {
            config,
            events: Vec::new(),
            nodes: HashMap::new(),
            slot_to_node: HashMap::new(),
            next_node_id: 0,
            current_frame: 0,
            frontier: SpiralIndex::new(0),
        }
    }

    /// Add a node to the mesh using SPIRAL self-assembly.
    pub fn add_node(&mut self) -> NodeId {
        let node_id = NodeId(self.next_node_id);
        self.next_node_id += 1;

        // Find next available slot
        let slot = self.find_next_slot();
        let coord = spiral_to_coord(slot);

        // Record join event
        self.events.push(MeshEvent::NodeJoined {
            node: node_id,
            slot,
            coord,
            frame: self.current_frame,
        });

        // Create node state
        let node_state = NodeState {
            id: node_id,
            slot,
            coord,
            connections: Vec::new(),
            is_valid: false,
        };

        // Establish connections to existing neighbors
        let neighbor_coords = Neighbors::of(coord);
        let mut connection_count = 0;

        for neighbor_coord in neighbor_coords {
            // Find node at this neighbor position (if any)
            if let Some(&neighbor_id) = self.find_node_at_coord(neighbor_coord) {
                // Establish connection
                self.events.push(MeshEvent::ConnectionEstablished {
                    from: node_id,
                    to: neighbor_id,
                    direction: 0, // Simplified - would compute actual direction
                    frame: self.current_frame,
                });

                // Mark as bidirectional (simplified - in real impl, neighbor confirms)
                self.events.push(MeshEvent::ConnectionConfirmed {
                    from: node_id,
                    to: neighbor_id,
                    frame: self.current_frame,
                });

                connection_count += 1;

                // Update both nodes' connection lists
                if let Some(neighbor) = self.nodes.get_mut(&neighbor_id) {
                    neighbor.connections.push(node_id);
                }
            }
        }

        // Check if node is now valid
        let existing_neighbors = connection_count;
        let threshold = validation_threshold(existing_neighbors);

        if connection_count >= threshold {
            self.events.push(MeshEvent::NodeValidated {
                node: node_id,
                connection_count,
                threshold,
                frame: self.current_frame,
            });
        }

        // Store node
        let mut final_state = node_state;
        final_state.is_valid = connection_count >= threshold;
        final_state.connections = self.nodes.values()
            .filter(|n| neighbor_coords.contains(&n.coord))
            .map(|n| n.id)
            .collect();

        self.nodes.insert(node_id, final_state);
        self.slot_to_node.insert(slot, node_id);

        // Update frontier if needed
        if slot.value() >= self.frontier.value() {
            self.frontier = SpiralIndex::new(slot.value() + 1);
        }

        self.current_frame += 1;
        node_id
    }

    /// Find the next available slot in SPIRAL order.
    fn find_next_slot(&self) -> SpiralIndex {
        // Simple: just use next in sequence (no gaps for now)
        SpiralIndex::new(self.nodes.len() as u64)
    }

    /// Find a node at the given coordinate.
    fn find_node_at_coord(&self, coord: HexCoord) -> Option<&NodeId> {
        self.nodes.values()
            .find(|n| n.coord == coord)
            .map(|n| &n.id)
    }

    /// Get all recorded events.
    pub fn events(&self) -> &[MeshEvent] {
        &self.events
    }

    /// Get the number of events recorded.
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Get the number of nodes in the mesh.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get a snapshot of the mesh at the current state.
    pub fn snapshot(&self) -> MeshSnapshot {
        let nodes: Vec<_> = self.nodes.values().cloned().collect();
        let connections: Vec<_> = self.nodes.values()
            .flat_map(|n| {
                n.connections.iter().map(move |&to| ConnectionState {
                    from: n.id,
                    to,
                    direction: 0,
                    is_bidirectional: true,
                })
            })
            .collect();

        let valid_count = nodes.iter().filter(|n| n.is_valid).count();
        let frontier_ring = self.frontier.ring();

        MeshSnapshot {
            frame: self.current_frame,
            nodes,
            connections,
            node_count: self.nodes.len(),
            valid_count,
            frontier_ring,
        }
    }

    /// Run assembly for N nodes.
    pub fn run_assembly(&mut self, count: usize) {
        for _ in 0..count {
            self.add_node();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulation_starts_empty() {
        let sim = Simulation::new(SimulationConfig::default());
        assert_eq!(sim.node_count(), 0);
        assert_eq!(sim.event_count(), 0);
    }

    #[test]
    fn first_node_is_origin() {
        let mut sim = Simulation::new(SimulationConfig::default());
        let node = sim.add_node();

        assert_eq!(node, NodeId(0));
        assert_eq!(sim.node_count(), 1);

        // First node should be at origin
        let state = sim.nodes.get(&node).unwrap();
        assert_eq!(state.slot, SpiralIndex::new(0));
        assert_eq!(state.coord, HexCoord::ORIGIN);
    }

    #[test]
    fn nodes_get_sequential_slots() {
        let mut sim = Simulation::new(SimulationConfig::default());

        for i in 0..10 {
            let node = sim.add_node();
            let state = sim.nodes.get(&node).unwrap();
            assert_eq!(state.slot.value(), i as u64);
        }
    }

    #[test]
    fn connections_established_to_neighbors() {
        let mut sim = Simulation::new(SimulationConfig::default());

        // Add 7 nodes (origin + 6 neighbors)
        sim.run_assembly(7);

        // Origin should have connections to ring-1 nodes
        let origin = sim.nodes.get(&NodeId(0)).unwrap();
        // Origin gets connections as neighbors join
        assert!(!origin.connections.is_empty());
    }

    #[test]
    fn snapshot_captures_state() {
        let mut sim = Simulation::new(SimulationConfig::default());
        sim.run_assembly(10);

        let snap = sim.snapshot();
        assert_eq!(snap.node_count, 10);
        assert!(snap.nodes.len() == 10);
    }

    #[test]
    fn events_recorded_for_each_join() {
        let mut sim = Simulation::new(SimulationConfig::default());
        sim.run_assembly(5);

        // Should have at least 5 NodeJoined events
        let join_events = sim.events().iter()
            .filter(|e| matches!(e, MeshEvent::NodeJoined { .. }))
            .count();

        assert_eq!(join_events, 5);
    }
}
