//! CPG Hashing - stable graph hashing for determinism validation
//!
//! Hash the entire CPG structure to detect unexpected changes.

use crate::cpg::model::CPG;
use sha2::{Digest, Sha256};

impl CPG {
    /// Compute SHA-256 hash of the entire CPG
    ///
    /// **Deterministic**: Same CPG → same hash
    pub fn compute_hash(&self) -> String {
        let mut hasher = Sha256::new();

        // Hash node count
        hasher.update(self.nodes.len().to_le_bytes());

        // Hash each node (in order)
        for node in &self.nodes {
            hasher.update(node.id.0.to_le_bytes());
            hasher.update(&[node.kind as u8]);
            hasher.update(node.source_range.start.to_le_bytes());
            hasher.update(node.source_range.end.to_le_bytes());
        }

        // Hash edge count
        hasher.update(self.edges.len().to_le_bytes());

        // Hash each edge (in order)
        for edge in &self.edges {
            hasher.update(edge.id.0.to_le_bytes());
            hasher.update(&[edge.kind as u8]);
            hasher.update(edge.from.0.to_le_bytes());
            hasher.update(edge.to.0.to_le_bytes());
        }

        format!("{:x}", hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpg::model::*;
    use crate::types::ByteRange;

    #[test]
    fn test_cpg_hash_empty() {
        let cpg = CPG::new();
        let hash1 = cpg.compute_hash();
        let hash2 = cpg.compute_hash();
        
        assert_eq!(hash1, hash2, "Same CPG produces same hash");
    }

    #[test]
    fn test_cpg_hash_determinism() {
        let mut cpg1 = CPG::new();
        cpg1.add_node(CPGNode::new(
            CPGNodeId(1),
            CPGNodeKind::Function,
            OriginRef::Function { function_id: crate::semantic::model::FunctionId(1) },
            ByteRange::new(0, 10),
        ));

        let mut cpg2 = CPG::new();
        cpg2.add_node(CPGNode::new(
            CPGNodeId(1),
            CPGNodeKind::Function,
            OriginRef::Function { function_id: crate::semantic::model::FunctionId(1) },
            ByteRange::new(0, 10),
        ));

        assert_eq!(cpg1.compute_hash(), cpg2.compute_hash());
    }
}
