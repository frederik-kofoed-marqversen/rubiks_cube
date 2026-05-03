## Functionality
Phase 2 indexers move tables and pruning tables
Parallelize table building
Mod 3 reduction of pruning tables
Symmetry reduction
Add commented methods to Cube 
Continuous solution improvement (keep searching for better solutions after first is found)
Flat const arrays instead of 2D in math might be faster/better

## Cleanup
Unify the three 4-edge indexers to reduce code duplication

## Other comments

use once_cell::sync::Lazy;

static TABLES: Lazy<Arc<KociembaTables>> = Lazy::new(|| {
    Arc::new(KociembaTables::load_or_build("tables.bin"))
});

Then:

let solver = KociembaSolver {
    tables: TABLES.clone(),
};

Could use crate serde for serialisation/deserialisation

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct KociembaTables {
    pub eo_move: MoveTable,
    // ... rest
}
