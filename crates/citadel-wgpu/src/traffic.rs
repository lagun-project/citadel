//! Traffic simulation and visualization.
//!
//! Simulates network traffic between mesh nodes:
//! - Random peer-to-peer traffic (unicast)
//! - Broadcast traffic (one-to-many)

use crate::hex_to_world;
use citadel_topology::Spiral3D;
use std::time::Instant;

/// A packet traveling between nodes.
#[derive(Clone, Debug)]
pub struct Packet {
    /// Source node position
    pub source: [f32; 3],
    /// Destination node position
    pub dest: [f32; 3],
    /// Progress along path (0.0 = source, 1.0 = destination)
    pub progress: f32,
    /// Packet type for coloring
    pub packet_type: PacketType,
    /// Time packet was created
    pub created: Instant,
}

/// Type of traffic for different visualizations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PacketType {
    /// Random peer-to-peer unicast
    Unicast,
    /// Broadcast to all neighbors
    Broadcast,
}

impl Packet {
    /// Create a new packet.
    pub fn new(source: [f32; 3], dest: [f32; 3], packet_type: PacketType) -> Self {
        Self {
            source,
            dest,
            progress: 0.0,
            packet_type,
            created: Instant::now(),
        }
    }

    /// Get current interpolated position.
    pub fn current_position(&self) -> [f32; 3] {
        let t = self.progress;
        [
            self.source[0] + (self.dest[0] - self.source[0]) * t,
            self.source[1] + (self.dest[1] - self.source[1]) * t,
            self.source[2] + (self.dest[2] - self.source[2]) * t,
        ]
    }

    /// Check if packet has arrived.
    pub fn arrived(&self) -> bool {
        self.progress >= 1.0
    }
}

/// Traffic simulation state.
pub struct TrafficSimulation {
    /// All node positions (cached for fast lookup)
    node_positions: Vec<[f32; 3]>,
    /// Number of currently visible/active nodes (traffic only between these)
    visible_nodes: u32,
    /// Active packets in flight
    pub packets: Vec<Packet>,
    /// Speed of packet travel (progress per second)
    pub packet_speed: f32,
    /// Statistics
    pub stats: TrafficStats,
    /// Random number generator state (simple LCG)
    rng_state: u64,
}

/// Traffic statistics.
#[derive(Default, Clone, Debug)]
pub struct TrafficStats {
    /// Total packets sent
    pub packets_sent: u64,
    /// Unicast packets sent
    pub unicast_sent: u64,
    /// Broadcast packets sent
    pub broadcast_sent: u64,
    /// Packets that arrived
    pub packets_delivered: u64,
}

impl TrafficSimulation {
    /// Create a new traffic simulation for the given node count.
    pub fn new(node_count: u32) -> Self {
        // Pre-compute all node positions
        let node_positions: Vec<[f32; 3]> = Spiral3D::take_slots(node_count as u64)
            .map(hex_to_world)
            .collect();

        Self {
            visible_nodes: node_count,
            node_positions,
            packets: Vec::with_capacity(10000),
            packet_speed: 2.0, // Takes 0.5 seconds to traverse
            stats: TrafficStats::default(),
            rng_state: 12345,
        }
    }

    /// Set the number of visible nodes (traffic only goes between visible nodes).
    pub fn set_visible_nodes(&mut self, count: u32) {
        self.visible_nodes = count.min(self.node_positions.len() as u32);
    }

    /// Simple random number generator.
    fn rand(&mut self) -> u64 {
        // LCG parameters from Numerical Recipes
        self.rng_state = self.rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.rng_state
    }

    /// Get a random visible node index.
    fn random_node(&mut self) -> usize {
        if self.visible_nodes == 0 {
            return 0;
        }
        (self.rand() as usize) % (self.visible_nodes as usize)
    }

    /// Spawn random unicast traffic.
    /// `intensity` is 0.0 to 1.0, controls how many packets to spawn.
    pub fn spawn_unicast(&mut self, intensity: f32) {
        if self.visible_nodes < 2 {
            return;
        }

        // Scale packets by intensity (0-500 packets based on intensity)
        let packet_count = (intensity * 500.0) as usize;

        for _ in 0..packet_count {
            let src_idx = self.random_node();
            let mut dst_idx = self.random_node();

            // Ensure different source and destination
            while dst_idx == src_idx {
                dst_idx = self.random_node();
            }

            let packet = Packet::new(
                self.node_positions[src_idx],
                self.node_positions[dst_idx],
                PacketType::Unicast,
            );

            self.packets.push(packet);
            self.stats.packets_sent += 1;
            self.stats.unicast_sent += 1;
        }
    }

    /// Spawn a single unicast packet from a random node to another random node.
    pub fn spawn_single_unicast(&mut self) {
        if self.visible_nodes < 2 {
            return;
        }

        let src_idx = self.random_node();
        let mut dst_idx = self.random_node();
        while dst_idx == src_idx {
            dst_idx = self.random_node();
        }

        let packet = Packet::new(
            self.node_positions[src_idx],
            self.node_positions[dst_idx],
            PacketType::Unicast,
        );

        self.packets.push(packet);
        self.stats.packets_sent += 1;
        self.stats.unicast_sent += 1;
    }

    /// Spawn broadcast traffic from random nodes.
    /// `intensity` is 0.0 to 1.0, controls how many broadcasts to spawn.
    pub fn spawn_broadcast(&mut self, intensity: f32) {
        if self.visible_nodes < 2 {
            return;
        }

        // Scale broadcasts by intensity (0-50 broadcast sources)
        let broadcast_count = (intensity * 50.0) as usize;

        for _ in 0..broadcast_count {
            let src_idx = self.random_node();
            let src_pos = self.node_positions[src_idx];

            // Get neighbors for this node (simulate 20-neighbor topology)
            // For simplicity, pick 20 random nearby nodes
            let neighbor_count = 20.min(self.visible_nodes as usize - 1);

            for _ in 0..neighbor_count {
                let dst_idx = self.random_node();
                if dst_idx != src_idx {
                    let packet = Packet::new(
                        src_pos,
                        self.node_positions[dst_idx],
                        PacketType::Broadcast,
                    );

                    self.packets.push(packet);
                    self.stats.packets_sent += 1;
                    self.stats.broadcast_sent += 1;
                }
            }
        }
    }

    /// Spawn a single broadcast from one random node to its neighbors.
    pub fn spawn_single_broadcast(&mut self) {
        if self.visible_nodes < 2 {
            return;
        }

        let src_idx = self.random_node();
        let src_pos = self.node_positions[src_idx];

        // Broadcast to up to 20 neighbors
        let neighbor_count = 20.min(self.visible_nodes as usize - 1);

        for _ in 0..neighbor_count {
            let dst_idx = self.random_node();
            if dst_idx != src_idx {
                let packet = Packet::new(
                    src_pos,
                    self.node_positions[dst_idx],
                    PacketType::Broadcast,
                );

                self.packets.push(packet);
                self.stats.packets_sent += 1;
                self.stats.broadcast_sent += 1;
            }
        }
    }

    /// Update all packets, removing arrived ones.
    pub fn update(&mut self, dt: f32) {
        // Update packet progress
        for packet in &mut self.packets {
            packet.progress += self.packet_speed * dt;
        }

        // Count and remove arrived packets
        let before = self.packets.len();
        self.packets.retain(|p| !p.arrived());
        let arrived = before - self.packets.len();
        self.stats.packets_delivered += arrived as u64;
    }

    /// Get all line vertices for rendering active packets.
    /// Returns pairs of (position, color) for line rendering.
    pub fn get_line_vertices(&self) -> Vec<LineVertex> {
        let mut vertices = Vec::with_capacity(self.packets.len() * 2);

        for packet in &self.packets {
            let color = match packet.packet_type {
                PacketType::Unicast => 0xFF00FFFF,  // Cyan (ABGR)
                PacketType::Broadcast => 0xFF00FF00, // Green (ABGR)
            };

            // Line from source to current position
            let current = packet.current_position();

            // Fade alpha based on progress
            let alpha = ((1.0 - packet.progress) * 255.0) as u32;
            let color_with_alpha = (color & 0x00FFFFFF) | (alpha << 24);

            vertices.push(LineVertex {
                position: packet.source,
                color: color_with_alpha,
            });
            vertices.push(LineVertex {
                position: current,
                color: color_with_alpha,
            });
        }

        vertices
    }

    /// Get point vertices for packet heads (for rendering as larger points).
    pub fn get_point_vertices(&self) -> Vec<LineVertex> {
        let mut vertices = Vec::with_capacity(self.packets.len());

        for packet in &self.packets {
            let color = match packet.packet_type {
                PacketType::Unicast => 0xFFFFFF00,  // Yellow (ABGR) - bright for visibility
                PacketType::Broadcast => 0xFF00FFFF, // Cyan (ABGR)
            };

            let current = packet.current_position();
            let alpha = ((1.0 - packet.progress * 0.5) * 255.0) as u32;
            let color_with_alpha = (color & 0x00FFFFFF) | (alpha << 24);

            vertices.push(LineVertex {
                position: current,
                color: color_with_alpha,
            });
        }

        vertices
    }

    /// Get number of active packets.
    pub fn active_packets(&self) -> usize {
        self.packets.len()
    }

    /// Clear all packets.
    pub fn clear(&mut self) {
        self.packets.clear();
    }
}

/// Vertex for line rendering.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LineVertex {
    pub position: [f32; 3],
    pub color: u32,
}

impl LineVertex {
    /// Get the vertex buffer layout for line vertices.
    pub fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<LineVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Color (packed u32)
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packet_interpolation() {
        let packet = Packet {
            source: [0.0, 0.0, 0.0],
            dest: [10.0, 0.0, 0.0],
            progress: 0.5,
            packet_type: PacketType::Unicast,
            created: Instant::now(),
        };

        let pos = packet.current_position();
        assert!((pos[0] - 5.0).abs() < 0.001);
    }

    #[test]
    fn traffic_simulation_creates_packets() {
        let mut sim = TrafficSimulation::new(100);
        sim.spawn_unicast(0.5);
        assert!(sim.packets.len() > 0);
    }

    #[test]
    fn line_vertex_size() {
        assert_eq!(std::mem::size_of::<LineVertex>(), 16);
    }
}
