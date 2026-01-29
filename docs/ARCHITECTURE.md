# VCR Architecture

**Abstract**
Valori Code Replay (VCR) is a deterministic, fail-closed static analysis kernel designed for high-assurance environments. This document outlines the system architecture, formally defining the separation between the User Space control plane and the Kernel Space data plane. The system prioritizes provenance and reproducibility over heuristic optimization.

## 1. System Topology

The architecture is strictly layered to enforce determinism boundaries.

```mermaid
%%{init: {'theme': 'base', 'themeVariables': { 'primaryColor': '#24292e', 'edgeLabelBackground':'#ffffff', 'tertiaryColor': '#f6f8fa', 'fontFamily': 'arial'}}}%%

graph TD
    %% --- Style Definitions ---
    classDef userSpace fill:#e3f2fd,stroke:#1565c0,stroke-width:2px,stroke-dasharray: 5 5;
    classDef kernelSpace fill:#f3e5f5,stroke:#7b1fa2,stroke-width:3px;
    classDef artifact fill:#eceff1,stroke:#455a64,stroke-width:2px;
    
    classDef process fill:#ffffff,stroke:#333,stroke-width:2px,color:#333;
    classDef data fill:#fff3e0,stroke:#ef6c00,stroke-width:2px,rx:5,ry:5;
    classDef critical fill:#fff8e1,stroke:#ff8f00,stroke-width:3px;
    classDef config fill:#000000,stroke:#ffffff,stroke-width:1px,color:#ffffff;

    %% --- User Space (Control Plane) ---
    subgraph UserSpace ["User Control Plane"]
        CLI(("\n    VCR CLI    \n")):::process
        
        subgraph ConfigScope ["Config (vcr.toml)"]
            direction TB
            Params1["Includes / Excludes"]:::config
            Params2["Thread Count"]:::config
            Params3["Memory Limits"]:::config
            Params4["Fail-Closed Mode"]:::config
        end
    end
    
    Lib["Rust Crate API\n(vcr)"]:::process

    %% --- Kernel Space (Data Plane) ---
    subgraph KernelSpace ["Deterministic Kernel Scope"]
        direction TB
        
        subgraph Epoch0 ["Ingestion Layer"]
            SourceFiles["Source Code"]:::data
            Parser[/"Incremental\nParser"/]:::process
            MMap["Memory Map\nArena"]:::data
            CanonicalHasher{{"Canonical\nHasher"}}:::critical
        end

        subgraph Epoch1 ["Semantic Construction"]
            AST["Abstract Syntax\nTree"]:::data
            SymTable["Symbol\nTable"]:::data
            GraphBuilder[/"Graph Builder"/]:::process
        end

        subgraph Epoch2 ["Graph Projection (CPG)"]
            UnifiedGraph[("Unified CPG")]:::critical
            
            subgraph Projections
                CFG["Control Flow"]:::data
                DFG["Data Flow"]:::data
                Taint["Taint Paths"]:::data
            end
        end

        subgraph Execution ["Query Execution"]
            Planner[/"Query Planner"/]:::process
            Tracer[/"Provenance Tracer"/]:::process
        end
    end

    %% --- Artifact Persistence ---
    subgraph Storage ["Persistence Layer"]
        Snapshot[("Snapshot Artifact\n(SHA-256 Verified)")]:::artifact
        JSON["Analysis Output\n(.json)"]:::artifact
    end

    %% --- Data Flow Relationships ---
    
    %% Input
    Config --> CLI
    Config --> Lib
    CLI -->|"Invokes"| Parser
    Lib -->|"Invokes"| Parser
    SourceFiles -->|"Read (mmap)"| MMap
    
    %% Phase 1
    MMap --> Parser
    Parser -->|"Delta Update"| AST
    MMap -->|"Bit-Exact Stream"| CanonicalHasher
    CanonicalHasher -.->|"Certifies"| UnifiedGraph

    %% Phase 2
    AST --> GraphBuilder
    SymTable --> GraphBuilder
    GraphBuilder -->|"Constructs"| UnifiedGraph

    %% Phase 3
    UnifiedGraph --- CFG
    UnifiedGraph --- DFG
    UnifiedGraph --- Taint

    %% Phase 4
    UnifiedGraph -->|"Query"| Planner
    Planner -->|"Trace"| Tracer
    Tracer -->|"Serialize"| JSON

    %% Storage
    UnifiedGraph ==>|"Serialize State"| Snapshot
    Snapshot ==>|"Hydrate (Replay)"| UnifiedGraph

    %% Validation Links (Dotted)
    CanonicalHasher -.->|"Verify Integrity"| Snapshot

    %% Formatting
    class UserSpace userSpace;
    class KernelSpace kernelSpace;
    class Storage artifact;
    
    linkStyle default stroke:black,stroke-width:2px,fill:none;
    linkStyle 9,15 stroke:#c62828,stroke-width:3px;
```

## 2. Component Formalism

### 2.1 The Trust Boundary
The system enforces a strict boundary between "Inputs" (potentially untrusted) and the "Unified CPG" (proven trusted).
*   **Canonical Hasher**: Serves as the distinct cryptographic gatekeeper. It computes `H(input)`.
*   **Fail-Closed Logic**: If any semantic construction results in a state $S$ such that $H(S) \neq H(input)$, the kernel panics immediately. This prevents "silent corruption".

### 2.2 Semantic Projections
The CPG is not a single graph but a unified projection of three mathematical structures:
1.  **AST ($T$)**: The hierarchical syntactic structure.
2.  **CFG ($G_c$)**: The directed graph of execution flow where edge $(u,v)$ implies control transfer.
3.  **DFG ($G_d$)**: The flow of data values, essential for Taint Analysis.

### 2.3 Persistence & Replayability
*   **Snapshots**: Are **isomorphic** to the in-memory state.
*   **Replay**: The operation `Load(Snapshot)` is guaranteed to yield a memory state identical to the original analysis epoch, preserving $O(1)$ query determinism over time.
