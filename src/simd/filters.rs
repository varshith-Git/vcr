//! SIMD filters (Step 4.2)
//!
//! **Mandatory**: Scalar fallback always available

use crate::cpg::model::{CPGNodeId, CPGNodeKind, CPGNode};

/// Filter nodes by kind (scalar baseline - always correct)
pub fn filter_by_kind_scalar(nodes: &[CPGNode], kind: CPGNodeKind) -> Vec<CPGNodeId> {
    nodes
        .iter()
        .filter(|n| n.kind == kind)
        .map(|n| n.id)
        .collect()
}

/// Filter nodes by kind (SIMD version - AVX2)
#[cfg(target_feature = "avx2")]
pub fn filter_by_kind_simd(nodes: &[CPGNode], kind: CPGNodeKind) -> Vec<CPGNodeId> {
    // Placeholder: would use AVX2 intrinsics here
    // For now, delegate to scalar (correct baseline)
    filter_by_kind_scalar(nodes, kind)
}

/// Filter nodes by kind (runtime dispatch)
pub fn filter_by_kind(nodes: &[CPGNode], kind: CPGNodeKind) -> Vec<CPGNodeId> {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            #[cfg(target_feature = "avx2")]
            return filter_by_kind_simd(nodes, kind);
        }
    }
    
    filter_by_kind_scalar(nodes, kind)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ByteRange;
    use crate::cpg::model::OriginRef;

    #[test]
    fn test_filter_scalar() {
        let nodes = vec![
            CPGNode::new(
                CPGNodeId(1),
                CPGNodeKind::Function,
                OriginRef::Function { function_id: crate::semantic::model::FunctionId(1) },
                ByteRange::new(0, 10),
            ),
            CPGNode::new(
                CPGNodeId(2),
                CPGNodeKind::CfgNode,
                OriginRef::Cfg { node_id: crate::semantic::model::NodeId(1) },
                ByteRange::new(10, 20),
            ),
        ];

        let funcs = filter_by_kind_scalar(&nodes, CPGNodeKind::Function);
        assert_eq!(funcs.len(), 1);
        assert_eq!(funcs[0], CPGNodeId(1));
    }

    #[test]
    fn test_simd_equals_scalar() {
        // BRUTAL: SIMD on/off must be identical
        let nodes = vec![
            CPGNode::new(
                CPGNodeId(1),
                CPGNodeKind::Function,
                OriginRef::Function { function_id: crate::semantic::model::FunctionId(1) },
                ByteRange::new(0, 10),
            ),
        ];

        let scalar_result = filter_by_kind_scalar(&nodes, CPGNodeKind::Function);
        let simd_result = filter_by_kind(&nodes, CPGNodeKind::Function);

        assert_eq!(scalar_result, simd_result, "SIMD must equal scalar");
    }
}
