## Functionality
Cleanup compute min distance
Remove let mut next = current; next.turn(mv); pattern?
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
