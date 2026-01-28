//! Query cost model (Step 4.3)

/// Query cost estimate
#[derive(Debug, Clone, Copy)]
pub struct QueryCost {
    /// Estimated node count in result
    pub node_count: usize,
    
    /// Average edge fanout
    pub edge_fanout: f64,
    
    /// Traversal depth
    pub traversal_depth: usize,
    
    /// Index selectivity (0.0 = all match, 1.0 = none match)
    pub index_selectivity: f64,
}

impl QueryCost {
    /// Create new cost estimate
    pub fn new(node_count: usize, edge_fanout: f64, traversal_depth: usize, index_selectivity: f64) -> Self {
        Self {
            node_count,
            edge_fanout,
            traversal_depth,
            index_selectivity,
        }
    }

    /// Estimate total cost (lower is better)
    pub fn total_cost(&self) -> f64 {
        (self.node_count as f64) 
            * self.edge_fanout 
            * (self.traversal_depth as f64) 
            * (1.0 - self.index_selectivity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_cost() {
        let cost = QueryCost::new(100, 2.5, 3, 0.1);
        assert!(cost.total_cost() > 0.0);
    }

    #[test]
    fn test_cost_comparison() {
        let cost1 = QueryCost::new(100, 1.0, 1, 0.5);
        let cost2 = QueryCost::new(10, 1.0, 1, 0.5);
        
        // Smaller node count = lower cost
        assert!(cost2.total_cost() < cost1.total_cost());
    }
}
