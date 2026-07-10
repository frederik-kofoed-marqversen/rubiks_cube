# Functionality
Make symmetry reduction tables always have identity symmetry for representatives! (optimisation/simplification)
Make UD move table using only phase 2 moves to save space
Add inverse cube method
Solve multiple cubes simultaneously using inverse cube and the C3 symmetry
Optimise valid_move function
Make UxD -> UD index lookup table, and make phase1 -> phase2 transition Cube struct free for speed
Add flag for optimal solutions found (full solve in phase1)

# Cleanup
Potentially make cube mul. repr. of moves
Flat const arrays instead of 2D in math might be faster/better
Make all symmetry functions be based on index rather than actual symmetries, consider where conjugation should be placed and how casting between cube group G and full cube move/symetry group Γ should be handled
Simplify indexer test macro
Add more docstrings (phase indexers)
Unify the three 4-edge indexers to reduce code duplication
Clenup Thislewaithe
