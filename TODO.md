## Functionality
Phase 2 indexers move tables and pruning tables
Parallelize table building
Mod 3 reduction of pruning tables
Symmetry reduction
Continuous solution improvement (keep searching for better solutions after first is found)

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