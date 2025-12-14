# Citadel Mesh Protocol: Complete Specification

## One Sentence

**The mesh topology IS the consensus state, verified through bilateral TGP proofs,
with security that scales from genesis trust to full BFT.**

---

## Core Principle

**THE MESH IS THE SOURCE OF TRUTH.**

There is no external oracle, coordinator, or database. The topology IS consensus.
Your slot = the connections you have. The mesh = the sum of all connections.

---

## The Final Simplification: TGP Over UDP

**"Connection" isn't a socket. It's a proof.**

```
OLD MODEL (TCP - WRONG):
├── SYN → SYN-ACK → ACK (open)
├── Keepalives to detect death
├── FIN → FIN-ACK → ACK (close)
├── Reconnection logic
├── "Is it open? Is it closed? Is it half-open?"
└── STATE MACHINE HELL

NEW MODEL (TGP over UDP - RIGHT):
├── QuadProof exists → accept packets from this peer
├── No QuadProof → drop packets
└── That's it. That's the whole thing.
```

### What This Eliminates

```
GONE:
├── Connection open/close state
├── Reconnection logic
├── Keepalive timers
├── Half-open detection
├── Connection timeout handling
├── "Is peer alive?" checks
└── TCP state machine (SYN, FIN, RST, etc.)

REMAINS:
├── QuadProof exists → accept packets
├── No QuadProof → drop packets
└── TGP flooding handles the rest
```

### Why TGP Makes This Work

```
TGP PROPERTIES:
├── Proofs are PERSISTENT (don't expire)
├── Proofs are SELF-AUTHENTICATING (no CA needed)
├── Proofs work over FLOODING (UDP broadcast)
├── Any copy of a proof suffices
└── "I know that you know that I know" survives packet loss

THEREFORE:
├── No need to "maintain" a connection
├── Just maintain the PROOF
├── Packets flow whenever network allows
├── No state beyond "proof exists"
```

### Implementation

```rust
struct MeshNode {
    // Not "connections" - just authorized peers
    authorized_peers: HashMap<PeerId, QuadProof>,
}

impl MeshNode {
    fn handle_udp_packet(&self, packet: &[u8], from: SocketAddr) {
        let peer_id = extract_peer_id(packet);

        // The ONLY check that matters
        if self.authorized_peers.contains_key(&peer_id) {
            self.process(packet);
        } else {
            // Drop. Not an error. Just unauthorized.
        }
    }
}
```

---

## THE VISION: Infinitely Dense Web

**Every peer is connected to every peer. The connection is a proof, not a socket. The overhead is zero.**

```
TRADITIONAL INTERNET:
├── N peers
├── Each peer maintains ~k connections (resource-limited)
├── k << N (because connections cost RAM, CPU, state)
├── To reach peer X: route through intermediaries
└── Connection overhead: O(k) per peer, O(Nk) total

TGP OVER UDP:
├── N peers
├── Each peer has proofs with ~M authorized peers
├── M can equal N (proofs cost ~256 bytes each, no state machine)
├── To reach peer X: send UDP packet directly (if proof exists)
└── Connection overhead: O(0)
```

### What "Connection" Becomes

```
OLD DEFINITION:
  connection = live bidirectional byte stream
  cost = socket + buffers + keepalives + state machine
  limit = ~10,000 per machine (generous)

NEW DEFINITION:
  connection = 256-byte QuadProof in a HashMap
  cost = 256 bytes storage, zero CPU when idle
  limit = ~4 million per GB of RAM
```

### SPIRAL Still Matters (But Differently)

```
SPIRAL'S ROLE:
├── NOT: "Who I'm connected to" (everyone is connected to everyone)
├── IS: "Who I route through for peers I don't have direct proof with"
├── IS: "Who I flood to for protocol messages"
└── IS: "Who validates my slot occupancy"

SPIRAL = routing optimization for the subset you actively flood to
PROOFS = authorization for the full set you CAN communicate with
```

### The Numbers

```
1 GB RAM for proofs = ~4 million authorized peers
Average proof establishment: one-time TGP handshake
Maintenance cost after establishment: 0
Packets from unknown peer: DROP (O(1) HashMap lookup)
Packets from authorized peer: ACCEPT
```

### Why This Changes Everything

```
BEFORE:
├── "How do we scale to 1M connections?"
├── "How do we handle reconnection?"
├── "How do we detect dead peers?"
└── HARD PROBLEMS

AFTER:
├── "1M connections" = 256 MB of proofs
├── "Reconnection" = send another UDP packet
├── "Dead peers" = who cares, no state to clean up
└── NON-PROBLEMS
```

**The internet was always this. TGP just adds cryptographic proof of bilateral intent.**

---

## THE KEY INSIGHT

```
"11/20" IS NOT THE MECHANISM.
"11/20" IS THE RESULT.

THE MECHANISM IS:
├── TGP at the base (bilateral consensus)
├── BFT emerges from TGP combinations
├── Threshold scales with network size
└── 11/20 is what BFT LOOKS LIKE at 20 neighbors

You don't "implement 11/20."
You implement TGP + scaling thresholds.
11/20 emerges at maturity.
```

---

## The Scaling Ladder

```
NODES    MECHANISM              THRESHOLD    HOW IT WORKS
─────────────────────────────────────────────────────────────
  2      TGP (bilateral)        1/1          Both agree or neither does
  3      TGP triad              2/3          Pairwise TGP, majority wins
 4-6     BFT emergence          ⌈n/2⌉+1      TGP pairs + deterministic tiebreaker
 7-11    Full BFT               2f+1         Threshold signatures
 12-20   Neighbor validation    scaled       Growing toward 11/20
 20+     Full SPIRAL            11/20        Mature mesh, all 20 neighbors exist
```

---

## Stage 1: Two Nodes (Pure TGP)

```
NODE A ←→ NODE B

MECHANISM:
├── Bilateral epistemic flooding
├── "I know that you know that I know"
├── Deterministic tiebreaker: lower(hash(A) XOR hash(tx))
├── OUTCOME: Both commit OR both abort
└── NO TIMEOUT. Existence-based.

SLOT IDENTITY:
├── A occupies slot 0 (origin)
├── B wants to be A's neighbor
├── TGP establishes: "We both agree B is at slot 1"
└── CONNECTION = SLOT (bilateral agreement)
```

---

## Stage 2: Three Nodes (TGP Triad)

```
    A
   / \
  B───C

MECHANISM:
├── A↔B: TGP pair
├── B↔C: TGP pair
├── A↔C: TGP pair
├── Each pair reaches bilateral consensus
├── TRIAD CONSENSUS: 2/3 pairs must agree
└── Deterministic tiebreaker resolves conflicts

SLOT IDENTITY:
├── Three slots: 0, 1, 2
├── Each node's position confirmed by 2 TGP agreements
└── Majority of pairs = consensus on topology
```

---

## Stage 3: BFT Emergence (4-11 Nodes)

```
     A
    /|\
   B-+-C
    \|/
     D

MECHANISM:
├── Every pair has TGP channel
├── Threshold signatures emerge: need k of n
├── k = 2f+1 where f = ⌊(n-1)/3⌋
├── THIS IS WHERE FLP BYPASS KICKS IN
│   └── "Do we have k signatures?" not "Did X respond?"
└── BFT consensus from TGP pairs + threshold

SLOT IDENTITY:
├── Node claims slot by establishing TGP with neighbors
├── Neighbors validate via their own TGP agreements
├── k neighbors confirming = you ARE that slot
└── Threshold scales with network size
```

---

## Stage 4: Full SPIRAL (12+ Nodes)

```
MECHANISM:
├── Each node has up to 20 theoretical neighbors
├── 11/20 threshold = BFT at full scale
├── Scaled validation: threshold(existing_neighbors)
│   └── If only 6 neighbors exist, need ceil(6 × 11/20) = 4
├── Security GROWS with network
└── Bootstrap is trusted, mature is BFT

SLOT IDENTITY:
├── You ARE slot N iff:
│   ├── You have TGP agreements with ≥threshold neighbors of N
│   └── Those neighbors acknowledge your direction as "toward N"
├── Pigeonhole: Each neighbor has ONE "toward N" direction
└── Can't have two nodes at same slot (directions exhausted)
```

---

## Why This Works: Security Scaling

```
SECURITY SCALING:
├── 2 nodes: Both must agree (trivial to attack, but also trivial network)
├── 3 nodes: 2/3 must agree (one Byzantine tolerated)
├── 7 nodes: 5/7 must agree (2 Byzantine tolerated)
├── 20 nodes: 11/20 must agree (9 Byzantine tolerated!)
└── Security GROWS with the network

BOOTSTRAP TRUST:
├── Genesis: Trust the first nodes (unavoidable)
├── Growth: Each new node validated by existing BFT
├── Maturity: Full 11/20 BFT, bootstrap irrelevant
└── "Security scales with network size"
```

---

## Foundation: Two Generals Protocol (TGP)

TGP is the atomic unit of consensus. Every connection in the mesh is a TGP bilateral coordination.

### TGP Properties

1. **Symmetric Outcomes**: Both parties either ATTACK (proceed) or ABORT together
2. **No Special Messages**: Any copy of a proof suffices (flooding-friendly)
3. **Bilateral Construction**: If one party can construct Q, so can the other
4. **Epistemic Fixpoint**: C → D → T → Q proof escalation

### TGP Phases

```
1. Commitment (C): Exchange signed intent to coordinate
2. Double (D): Prove receipt of counterparty's commitment
3. Triple (T): Prove knowledge of counterparty's double proof
4. Quad (Q): Achieve epistemic fixpoint - coordination complete
```

### The Optimized Handshake: 4 Packets, Forever Free

**Four packets to meet. Zero packets to reconnect. The handshake is forever.**

```
PACKET 1 (A→B): C_A                         # A's commitment
PACKET 2 (B→A): C_B + D_B                   # B's commitment + proof of A's
PACKET 3 (A→B): D_A + T_A                   # A's double + triple
PACKET 4 (B→A): T_B + Q_B                   # B's triple + quad

RESULT: Both have QuadProof. Forever.
```

### State Transitions

```
BEFORE HANDSHAKE:
├── Peer X sends packet → DROPPED (no proof)
├── Door is "open" (UDP accepts anything)
├── But we drop unauthorized
└── Cost: 0 (just a HashMap miss)

DURING HANDSHAKE (4 packets):
├── Packet 1: A→B commitment
├── Packet 2: B→A commitment + double
├── Packet 3: A→B double + triple
├── Packet 4: B→A triple + quad
└── Cost: 4 packets, ~2 RTT

AFTER HANDSHAKE:
├── Peer X sends packet → ACCEPTED (proof exists)
├── Data flows immediately
├── No handshake ever again
└── Cost: 0 forever
```

### Compare to TCP

```
TCP (Every new connection):
├── SYN →
├── ← SYN-ACK
├── ACK →
├── ...now you can send data
└── EVERY. SINGLE. TIME.

TGP (Once per peer, ever):
├── 4 packets total
├── Then NEVER AGAIN
├── Future contact: just send
└── Half-RTT to start transfer (they already have proof)
```

### The Half-RTT Instant Start

After QuadProof exists, starting a transfer takes half an RTT:

```
A wants to send data to B:
├── A sends: DATA packet
├── B receives: checks proof, ACCEPTS
├── Transfer starts
└── HALF RTT. No handshake needed.

Because:
├── Authorization already proven (QuadProof)
├── No need to re-establish anything
├── Just send
└── They'll accept
```

### The Math

```
TRADITIONAL:
├── N peers you talk to regularly
├── Each needs 3-way handshake per session
├── K sessions per day
├── Cost: N × K × 1.5 RTT per day

TGP:
├── N peers you ever talk to
├── Each needs 4 packets ONCE
├── Cost: N × 2 RTT TOTAL, LIFETIME
├── Then: 0 RTT forever
```

**TCP reconnects every session. TGP connects once, permanently.**

### Adaptive Flooding

TGP uses continuous adaptive flooding, not request-response:

- **Drip mode**: ~1 packet/300s when idle (keepalive)
- **Burst mode**: Up to 50MB/s+ when active coordination needed

The rate ramps up instantly when needed and slowly decays to drip mode.

---

## Deterministic Tiebreaker

When two nodes contest the same resource (slot, direction), the winner is determined by:

```rust
// The transaction is the full commitment being contested
priority = hash(blake3(peer_id) XOR blake3(transaction))
// Lower hash wins

// Example for slot claims:
let transaction = format!("slot_claim:{}:{}:{}:{}", index, coord.q, coord.r, coord.z);
let priority = blake3::hash(&xor(blake3(peer_id), blake3(transaction)));
```

### Why This Is Ungameable

1. **Can't predict transaction**: When generating your peer_id, you don't know what you'll be contesting
2. **Includes full context**: Not just slot index, but the entire commitment
3. **Deterministic**: All honest nodes compute the SAME priority for any (peer_id, transaction) pair
4. **No timestamps**: No "first writer wins" - arrival order doesn't matter
5. **No coordinator**: Each node computes locally
6. **Grinding resistant**: Transaction includes context you can't control

---

## Timeouts

```
TIMEOUTS:
├── Used for: Resource cleanup (free stale coordinators)
├── NOT used for: Consensus decisions
├── On timeout: ABORT (safe default)
└── "Silence means nothing happened, not failure"
```

Timeouts don't affect correctness, only resource management. If a TGP coordination
times out, the correct action is ABORT (safe state), not "assume success."

---

## Consensus Threshold Scaling

The number of TGP connections required for slot occupancy scales with mesh size:

| Mesh Size | Threshold | Description |
|-----------|-----------|-------------|
| 1         | 1         | Genesis node auto-occupies slot 0 |
| 2         | 2         | Pure TGP: 2/2 bilateral |
| 3         | 2         | Triad: 2/3 (can tolerate 1 fault) |
| 4         | 3         | BFT emerges: 3/4 |
| 5-6       | 4         | Growing BFT |
| 7-9       | 5         | Approaching 2/3 |
| 10-14     | 7         | 2/3 + 1 for larger groups |
| 15-19     | 9         | Approaching full mesh |
| 20+       | 11        | Full BFT: 11/20 |

**This is NOT arbitrary.** These are the minimum thresholds for Byzantine fault tolerance at each scale.

---

## Slot Occupancy

### The Truth

```
YOU DON'T "CLAIM" A SLOT.
YOU DON'T "GET" A SLOT.
YOU **BECOME** A SLOT BY HAVING THE CONNECTIONS.
```

### How It Works

A node occupies slot N if and only if:

1. It has `threshold` or more successful TGP connections
2. Those connections are to nodes at slot N's theoretical neighbor positions
3. Each connection is bidirectional (both parties have QuadProofs)

### Pigeonhole Exclusivity

Each neighbor has exactly ONE direction slot pointing toward position N.
That direction holds exactly ONE peer (enforced by TGP).

If node X has enough directions toward slot N → node Y cannot have them.
If Y cannot have them → Y cannot reach threshold.
If Y cannot reach threshold → Y is NOT at slot N.

**No explicit rejection needed. The topology makes conflicts IMPOSSIBLE.**

---

## Join Sequence

### Step 1: Learn the Mesh

Query bootstrap nodes or existing connections to get the mesh directory:
- List of (peer_id, address, slot_index) tuples
- Current mesh size for threshold calculation

### Step 2: Calculate Target Slot

```rust
target_slot = first_unclaimed_slot_in_spiral_order(directory)
```

### Step 3: Identify Theoretical Neighbors

```rust
neighbor_coords = SPIRAL.neighbors(target_slot)  // 20 coordinates
existing_neighbors = directory.filter(|n| neighbor_coords.contains(n.coord))
```

### Step 4: Attempt TGP Connections

For each existing neighbor:

```rust
// Create TGP coordinator for this connection
let coordinator = PeerCoordinator::new(
    my_keypair,
    neighbor.public_key,
    CoordinatorConfig::initiator()
        .with_commitment(format!("mesh_connection:{}:{}", my_slot, neighbor.slot))
);

// Exchange proofs until coordinated or timeout
coordinator.set_active(true);
while !coordinator.is_coordinated() && !coordinator.has_timed_out() {
    // Send proofs
    if let Some(messages) = coordinator.poll() {
        send_to(neighbor, messages);
    }
    // Receive proofs
    for msg in receive_from(neighbor) {
        coordinator.receive(&msg);
    }
}
```

### Step 5: Evaluate Threshold

```rust
successful_connections = coordinators.filter(|c| c.is_coordinated()).count();
threshold = consensus_threshold(mesh_size);

if successful_connections >= threshold {
    // SUCCESS: We occupy the target slot
    self.slot = Some(target_slot);
    flood_announcement(SlotClaim { slot: target_slot, peer_id: my_id });
} else {
    // RETRY: Try next slot in SPIRAL order
    try_join_at(target_slot + 1);
}
```

---

## Bootstrap Nodes

### What Bootstrap Does

1. **Answers queries**: New node connects, asks "who's in the mesh?"
2. **Provides directory**: Returns list of (peer_id, address, slot) tuples
3. **Disconnects**: Connection is temporary, not permanent

### What Bootstrap Does NOT Do

- Does NOT count toward your slot connections
- Does NOT validate your slot occupancy
- Does NOT maintain persistent connections
- Is NOT part of the mesh topology

### Bootstrap Capacity

Since bootstrap just answers queries:
- Can handle unlimited queries (no connection limit)
- Stateless or near-stateless
- Can be replicated for availability

---

## SPORE's Role

SPORE synchronizes the DIRECTORY, not the consensus.

### What SPORE Floods

- Slot claims: (peer_id, slot_index, coord)
- Peer info: (addresses, capabilities, last_seen)
- NOT the TGP proofs themselves (those are bilateral)

### Why

- TGP proofs are bilateral (between two specific nodes)
- But KNOWLEDGE of who's where is global
- SPORE makes the directory eventually consistent
- XOR cancellation means converged nodes sync at near-zero cost

---

## Emergent Properties

From TGP + Deterministic Tiebreaker + Adaptive Threshold:

1. **No coordinator**: TGP is peer-to-peer
2. **No timestamps**: Hash priority, not arrival order
3. **No FWW**: Deterministic tiebreaker
4. **Byzantine tolerant**: Threshold scales with BFT requirements
5. **Self-healing**: Wrong connections can't reach threshold
6. **Provable**: Slot occupancy = existence of TGP QuadProofs

---

## The Beautiful Recursion

```
TGP proves bilateral connections.
Connections define slot occupancy.
Slot occupancy defines the mesh.
The mesh IS the consensus.

No external oracle.
The mesh computes itself.
```

---

## Implementation Files

- `citadel-protocols/src/coordinator.rs` - TGP PeerCoordinator
- `citadel-protocols/src/spore_sync.rs` - SPORE sync manager
- `citadel-lens/src/mesh.rs` - Mesh service (to be fixed)
- `citadel-topology/src/neighbors.rs` - SPIRAL neighbor calculation

## Proof Files

- `proofs/CitadelProofs/Convergence.lean` - Slot occupancy through connections
- `proofs/CitadelProofs/TwoHopKnowledge.lean` - Knowledge modes (Mix/Smart/Full)
- `proofs/CitadelProofs/Spore.lean` - SPORE XOR cancellation
- `proofs/CitadelProofs/Spiral3D.lean` - 3D shell enumeration
