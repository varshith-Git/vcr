//! Query planner (Step 4.3)
//!
//! **Reorder queries, never reinterpret**

use crate::optimizer::cost::QueryCost;
use std::collections::HashMap;

/// Query hash (query + graph hash)
pub type QueryHash = u64;

/// Cached query plan
#[derive(Debug, Clone)]
pub struct CachedPlan {
    pub query_hash: QueryHash,
    pub estimated_cost: QueryCost,
}

/// Query planner with caching
pub struct QueryPlanner {
    /// Plan cache: (query hash, graph hash) → plan
    cache: HashMap<QueryHash, CachedPlan>,
}

impl QueryPlanner {
    /// Create new planner
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Get cached plan
    pub fn get_plan(&self, query_hash: QueryHash) -> Option<&CachedPlan> {
        self.cache.get(&query_hash)
    }

    /// Cache a plan
    pub fn cache_plan(&mut self, query_hash: QueryHash, cost: QueryCost) {
        self.cache.insert(query_hash, CachedPlan {
            query_hash,
            estimated_cost: cost,
        });
    }

    /// Clear cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_planner_cache() {
        let mut planner = QueryPlanner::new();
        let cost = QueryCost::new(100, 1.0, 1, 0.5);
        
        planner.cache_plan(12345, cost);
        assert!(planner.get_plan(12345).is_some());
        assert!(planner.get_plan(99999).is_none());
    }
}
