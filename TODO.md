# Functionality
Symmetry reduction
Add commented methods to Cube 
Continuous solution improvement (keep searching for better solutions after first is found)
Flat const arrays instead of 2D in math might be faster/better

# Overview
## Phase 1:
Indices: eos, slice, co

eos = edge_orientation x slice
eos symmetry reduced to eos_class via symmetry s
co_eq = s(corner_orientation)

prune table using eos_class x co_eq

## Phase 2:
Indices: udep, slice, cp

cps = corner_permutation x slice_sorted
prune table using cps

need function to compute udp from uep, dep
need a move table for udep = ud_edges_permutation

cp symmetry reduced to cp_class via symmetry s
udep_eq = s(udep)

prune tabel using cp_class x edep_eq

# Cleanup
Unify the three 4-edge indexers to reduce code duplication

# Other comments

use once_cell::sync::Lazy;

static TABLES: Lazy<Arc<KociembaTables>> = Lazy::new(|| {
    Arc::new(KociembaTables::load_or_build("tables.bin"))
});

Then:

let solver = KociembaSolver {
    tables: TABLES.clone(),
};
