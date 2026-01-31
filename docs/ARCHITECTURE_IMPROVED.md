# VCR Architecture - Improved Diagram for Research Paper

This document contains simplified, paper-friendly versions of the VCR architecture diagram.

## Version 1: High-Level System Overview (Recommended for Paper)

This version emphasizes the three-layer architecture and trust boundary.

```mermaid
%%{init: {'theme': 'base', 'themeVariables': { 'primaryColor': '#f5f5f5', 'primaryTextColor': '#000', 'lineColor': '#333', 'fontSize': '14px'}}}%%

graph TB
    %% Style definitions
    classDef userLayer fill:#e3f2fd,stroke:#1565c0,stroke-width:2px
    classDef kernelLayer fill:#fff3e0,stroke:#ef6c00,stroke-width:3px
    classDef persistLayer fill:#f3e5f5,stroke:#7b1fa2,stroke-width:2px
    classDef process fill:#fff,stroke:#333,stroke-width:2px
    classDef trustBoundary stroke:#d32f2f,stroke-width:4px,stroke-dasharray:5 5
    
    %% User Space
    subgraph User["👤 User Space (Control Plane)"]
        CLI[CLI Interface]
        Config[Configuration<br/>vcr.toml]
    end
    
    %% Trust Boundary
    TB{{"🔒 Trust Boundary<br/>(Canonical Hasher)"}}:::trustBoundary
    
    %% Kernel Space
    subgraph Kernel["🔬 Deterministic Kernel (Data Plane)"]
        direction TB
        
        subgraph E1["Epoch 1: Parse"]
            Parse[Tree-sitter<br/>Parser]
            AST[Abstract Syntax Tree]
        end
        
        subgraph E2["Epoch 2: Semantic"]
            CFG[Control Flow<br/>Graph]
            DFG[Data Flow<br/>Graph]
        end
        
        subgraph E3["Epoch 3: Unified CPG"]
            CPG[(Code Property<br/>Graph)]
            Hash{{SHA-256<br/>Hash}}
        end
        
        Query[Query Engine]
    end
    
    %% Persistence
    subgraph Persist["💾 Persistence Layer"]
        Snapshot[Snapshot<br/>Storage]
        Results[JSON Output]
    end
    
    %% Flows
    CLI --> Config
    Config --> TB
    TB -->|"Verified Input"| Parse
    
    Parse --> AST
    AST --> CFG
    AST --> DFG
    
    CFG --> CPG
    DFG --> CPG
    
    CPG --> Hash
    Hash -.->|Verify| CPG
    
    CPG --> Query
    Query --> Results
    
    CPG <==>|Save/Load| Snapshot
    Hash -.->|Verify| Snapshot
    
    %% Apply styles
    class User userLayer
    class Kernel kernelLayer
    class Persist persistLayer
    class CLI,Config,Parse,AST,CFG,DFG,Query process
```

**Key Improvements:**
- Clear separation of three layers
- Trust boundary explicitly shown
- Epoch flow is obvious (E1 → E2 → E3)
- Reduced visual clutter
- Grayscale-friendly colors

---

## Version 2: Detailed Data Flow (For Technical Readers)

This version shows more implementation details while maintaining clarity.

```mermaid
%%{init: {'theme': 'base', 'themeVariables': { 'fontSize': '13px'}}}%%

graph LR
    %% Style definitions
    classDef input fill:#e8f5e9,stroke:#2e7d32,stroke-width:2px
    classDef epoch fill:#fff9c4,stroke:#f57c00,stroke-width:2px
    classDef output fill:#e1f5fe,stroke:#0277bd,stroke-width:2px
    classDef verify fill:#ffebee,stroke:#c62828,stroke-width:3px
    
    %% Input
    Source[Source Code<br/>Files]:::input
    
    %% Epoch 1
    subgraph EP1["Parse Epoch"]
        direction TB
        Mmap[Memory Map]
        TreeSitter[Tree-sitter<br/>Incremental Parser]
        Trees[Syntax Trees]
    end
    
    %% Epoch 2
    subgraph EP2["Semantic Epoch"]
        direction TB
        SymTab[Symbol Table]
        CFGBuild[CFG Builder]
        DFGBuild[DFG Builder]
    end
    
    %% Epoch 3
    subgraph EP3["CPG Epoch"]
        direction TB
        Merge[Graph Merger]
        Sort[Deterministic Sort]
        CPGFinal[Unified CPG]
    end
    
    %% Verification
    Hasher{{Canonical<br/>SHA-256}}:::verify
    
    %% Output
    Snap[(Snapshot)]:::output
    JSON[Analysis<br/>Results]:::output
    
    %% Flows
    Source --> Mmap
    Mmap --> TreeSitter
    TreeSitter --> Trees
    
    Trees --> SymTab
    Trees --> CFGBuild
    Trees --> DFGBuild
    
    CFGBuild --> Merge
    DFGBuild --> Merge
    SymTab --> Merge
    
    Merge --> Sort
    Sort --> CPGFinal
    
    CPGFinal --> Hasher
    Hasher -.->|Verify| CPGFinal
    
    CPGFinal --> Snap
    CPGFinal --> JSON
    
    Snap -.->|Hash Check| Hasher
    
    %% Apply styles
    class EP1,EP2,EP3 epoch
```

**Use this for:**
- Technical appendix
- Detailed implementation discussion

---

## Version 3: Epoch Pipeline (Simplest - Great for Abstract/Intro)

```mermaid
graph LR
    classDef epoch fill:#e3f2fd,stroke:#1565c0,stroke-width:2px
    classDef hash fill:#fff3e0,stroke:#ef6c00,stroke-width:3px
    
    Source[Source Code]
    
    E1[Parse<br/>Epoch]:::epoch
    E2[Semantic<br/>Epoch]:::epoch
    E3[CPG<br/>Epoch]:::epoch
    
    H{{Hash<br/>Verify}}:::hash
    
    Output[Snapshot +<br/>Results]
    
    Source ==> E1
    E1 ==> E2
    E2 ==> E3
    E3 ==> H
    H -.-> E3
    E3 ==> Output
```

**Use this for:**
- Paper introduction
- Quick concept explanation
- Presentations

---

## Version 4: Parallel Execution Model (For Implementation Section)

```mermaid
%%{init: {'theme': 'base'}}%%

graph TD
    classDef task fill:#e1f5fe,stroke:#0277bd,stroke-width:2px
    classDef compute fill:#fff9c4,stroke:#f57c00,stroke-width:2px
    classDef commit fill:#e8f5e9,stroke:#2e7d32,stroke-width:2px
    
    Plan[Execution Plan<br/>Sequential Task IDs]
    
    subgraph Parallel["⚡ Parallel Compute Phase"]
        T1[Task 1]:::task
        T2[Task 2]:::task
        T3[Task 3]:::task
        TN[Task N]:::task
    end
    
    Buffer[Result Buffer<br/>Unsorted]:::compute
    
    Sort[Sort by Task ID]:::commit
    
    subgraph Serial["🔒 Serial Commit Phase"]
        C1[Commit 1]:::commit
        C2[Commit 2]:::commit
        C3[Commit 3]:::commit
    end
    
    CPG[(CPG<br/>Deterministic)]
    
    Plan --> T1 & T2 & T3 & TN
    T1 & T2 & T3 & TN --> Buffer
    Buffer --> Sort
    Sort --> C1
    C1 --> C2
    C2 --> C3
    C3 --> CPG
```

**Use this for:**
- Explaining parallel ≡ serial theorem
- Implementation section of paper

---

## Recommendation for Your Paper

**For the main architecture section (Section 4), use Version 1** - it's clean, professional, and emphasizes the key architectural principles without overwhelming detail.

**For the implementation section (Section 5), add Version 4** - it shows how you achieve deterministic parallelism.

**For the introduction or abstract**, consider using **Version 3** - it's the simplest and gives readers an immediate understanding of the epoch pipeline.

Would you like me to:
1. Update your ARCHITECTURE.md with one of these improved versions?
2. Create a standalone diagram file for the paper?
3. Export these as images (PNG/PDF) for direct use in LaTeX?
