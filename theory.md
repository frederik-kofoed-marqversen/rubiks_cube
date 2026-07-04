Currently this is just a refinement of my formalism by ChatGPT should be reread!!!
Need to add description of what is 
    cube1 * cube2 (combine scrambles...)
    s * cube * s^-1 (consider applying to solved cube. Then rotate cube by symmetry. Apply scramble. Rotate cube back. So it is like applying the scramble facing a different face than usual)

# Formal treatment of symmetry reduction for Kociemba distance tables

## 1. Cube states and indexers

Let G be the Rubik's cube group: the set of all cube states reachable from the solved cube by legal moves.

A cube state is:

    c ∈ G

The group operation is composition of transformations:

    c2 c1

meaning that c1 is applied first and c2 afterwards.

Let:

    M

be the set of cube moves. Each move is an element of G and acts on cube states by composition.

An indexer is a surjective function:

    I : G -> {0,...,|I|-1}

where |I| is the number of indices represented.

A move table for an indexer is a function:

    M_I : {0,...,|I|-1} × M -> {0,...,|I|-1}

such that:

    M_I(I(c), m) = I(mc)

provided that the indexer is compatible with the move action.

For two indexers:

    I1 : G -> A
    I2 : G -> B

we define the product indexer:

    I = I1 ⊗ I2

by:

    I(c) = (I1(c), I2(c))

or numerically:

    I(c) = I1(c)|I2| + I2(c)

with:

    |I| = |I1||I2|

If I is one-to-one, it is a complete coordinate representation of the cube state.

---

## 2. Distance tables

Let:

    d : G -> N

be the optimal distance from a state to solved.

A distance table for indexer I is:

    D_I : {0,...,|I|-1} -> N

with:

    D_I(I(c)) = d(c)

This is well-defined if all states with the same index have the same distance.

For a complete coordinate system:

    |D_I| = |I|

---

# 3. The full symmetry group

The cube has spatial symmetries that are not necessarily reachable cube states.

Let:

    Γ

be the full symmetry group of the cube.

Γ contains:

- rotations
- reflections

The cube group is a subgroup:

    G ⊂ Γ

but not every element of Γ is in G.

For example, a 90 degree physical rotation of the solved cube is a valid spatial symmetry, but it may not correspond to a reachable cube state because of permutation parity constraints.

Therefore:

    s ∈ Γ

does not imply:

    s ∈ G

This distinction is important.

A symmetry is not a cube state. It is a transformation of the coordinate system.

---

# 4. Action of symmetries on cube states

Although a symmetry itself may not be a reachable cube state, it acts on cube states.

The action is defined by conjugation:

    φ_s(c) = s c s^-1

where:

    s ∈ Γ
    c ∈ G

Because G is a normal subgroup of Γ:

    s c s^-1 ∈ G

Therefore symmetry conjugation maps cube states to cube states:

    Γ × G -> G

given by:

    (s,c) -> s c s^-1

This has the expected properties:

Solved remains solved:

    s e s^-1 = e

and inverse symmetries undo the transformation:

    s^-1(scs^-1)s = c

The distance function is invariant:

    d(c) = d(scs^-1)

because symmetries do not change the shortest solution length.

---

# 5. Symmetry-reduced indices

A symmetry orbit of a cube state is:

    [c] = { s(c) | s ∈ Γ }

Two states belong to the same orbit if:

    c1 ~ c2

when:

    exists s ∈ Γ such that:

        c2 = s c1 s^-1

The symmetry-reduced index is the quotient:

    S[I] : G -> G/Γ

with:

    S[I](c1) = S[I](c2)

iff:

    exists s ∈ Γ:

        c2 = s c1 s^-1

The size reduction is approximately:

    |S[I]| ≈ |I| / |Γ|

for generic states.

The exact reduction depends on orbit sizes because some states have non-trivial stabilizers.

---

# 6. Why reducing only one factor of a product index works

Suppose:

    I = I1 ⊗ I2

is a complete coordinate system.

The full distance table has size:

    |D_I| = |I1||I2|

A full symmetry reduction gives:

    |D_S[I]| ≈ |I1||I2| / |Γ|

but requires knowing, for every state index, which symmetry maps it to its representative.

This requires roughly:

    |I1||I2|

additional information, removing the memory benefit.

Instead, reduce only one factor:

    I1 -> S[I1]

and store:

- a map from I1 indices to symmetry classes
- the symmetry that maps a state to its representative

The distance table becomes:

    D(S[I1], I2)

with size:

    |S[I1]||I2|

approximately:

    |I1||I2| / |Γ|

The additional symmetry lookup cost is approximately:

    O(|I1|)

so the total memory is:

    O(|I1|) + |I1||I2|/|Γ|

The first term is small compared with the reduced distance table.

Therefore symmetry reduction gives a real memory reduction.

---

# 7. Why conjugation must be applied to the other coordinates

Suppose:

    I = I1 ⊗ I2

and we reduce only I1.

For a cube state c, choose a symmetry s such that:

    S[I1](c) = I1(scs^-1)

The distance is unchanged:

    d(c) = d(scs^-1)

Therefore:

    D(I1(c), I2(c))

can be looked up as:

    D(S[I1](c), I2(scs^-1))

The second coordinate must also be transformed by the same symmetry.

It is not enough to transform only I1, because the pair:

    (I1(c), I2(c))

represents a complete cube state.

The transformed coordinates must still describe the same transformed cube:

    scs^-1

This is why Kociemba computes the second coordinate using the conjugated cube state.

---

# 8. Practical consequence

The correct conceptual separation is:

Cube states:

    G

Symmetries:

    Γ

with operations:

    Cube × Cube -> Cube

for cube composition,

    Symmetry × Symmetry -> Symmetry

for symmetry composition,

and:

    Symmetry acts on Cube by:

        c -> scs^-1

for symmetry reduction.

This separation avoids treating unreachable spatial transformations as ordinary cube states while still allowing the symmetry reductions required by Kociemba's algorithm.