# Functionality
# Rebuild
Rewrite IDA* solver
# Further work
Make UD move table using only phase 2 moves to save space
Continuous solution improvement (keep searching for better solutions after first is found)
Add inverse cube method
Solve multiple cubes simultaneously using inverse cube and the C3 symmetry
Phase2Indexer2 could also use symmetry reduction on cp to reduce space (cp class and sym should only be computed once for both indexers though)
Optimise valid_move function
Make UxD -> UD index lookup table, and make phase1 -> phase2 transition Cube struct free for speed
Potentially make cube mul. repr. of moves
Flat const arrays instead of 2D in math might be faster/better

# Cleanup
Make trait CubeRepresentation wich is Moveable + from Cube - use that to get solved for PruneTable::build
Properly compute phase state indexer SIZEs instead of hard coded 
Add more docstrings (phase indexers)
Refactor and proper general implementation
Unify the three 4-edge indexers to reduce code duplication
Phase indexers can potentially get from_index implementations

# Other comments

use once_cell::sync::Lazy;

static TABLES: Lazy<Arc<KociembaTables>> = Lazy::new(|| {
    Arc::new(KociembaTables::load_or_build("tables.bin"))
});

Then:

let solver = KociembaSolver {
    tables: TABLES.clone(),
};
