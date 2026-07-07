# Functionality
Make symmetry reduction tables always have identity symmetry for representatives! (optimisation/simplification)
Make UD move table using only phase 2 moves to save space
Continuous solution improvement (keep searching for better solutions after first is found)
Add inverse cube method
Solve multiple cubes simultaneously using inverse cube and the C3 symmetry
Phase2Indexer2 could also use symmetry reduction on cp to reduce space (cp class and sym should only be computed once for both indexers though). Investigate if this is worth it. (add a memory count function)
Optimise valid_move function
Make UxD -> UD index lookup table, and make phase1 -> phase2 transition Cube struct free for speed

# Cleanup
Potentially make cube mul. repr. of moves
Flat const arrays instead of 2D in math might be faster/better
Make all symmetry functions be based on index rather than actual symmetries, consider where conjugation should be placed and how casting between cube group G and full cube move/symetry group Γ should be handled
Simplify indexer test macro
Add more docstrings (phase indexers)
Unify the three 4-edge indexers to reduce code duplication
Clenup Thislewaithe
