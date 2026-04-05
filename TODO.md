Implement Phase 1 IDA*
Verify Phase 1 works (random cubes)
Implement Phase 2 indexers
Build Phase 2 move tables
Build Phase 2 pruning
Implement Phase 2 search
Finally:
Combine into full solver
Optimize
(Optional) symmetry reduction





## Later

use once_cell::sync::Lazy;

static TABLES: Lazy<Arc<KociembaTables>> = Lazy::new(|| {
    Arc::new(KociembaTables::load_or_build("tables.bin"))
});

Then:

let solver = KociembaSolver {
    tables: TABLES.clone(),
};