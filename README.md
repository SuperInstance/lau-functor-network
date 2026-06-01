# lau-functor-network

**Category-theoretic framework for composing the 320+ crate SuperInstance ecosystem.**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Tests: 102](https://img.shields.io/badge/tests-102-brightgreen.svg)]()

---

## What This Does

`lau-functor-network` treats every crate in the SuperInstance ecosystem as an **object in a category** and every dependency as a **morphism**. It then applies the full machinery of category theory — functors, natural transformations, adjunctions, monads, limits, colimits, Kan extensions, Yoneda lemma, and sheaf theory — to analyze, compose, and reason about crate relationships.

The crate provides:

- **Categories** — objects (crates), morphisms (dependencies), identity laws, associativity, composition
- **Functors** — structure-preserving maps between categories of crates
- **Natural Transformations** — morphisms between functors (comparing two ways of mapping subcategories)
- **Adjunctions** — pairs of functors (free/forgetful, product/diagonal, etc.)
- **Monads** — computational effects modeled categorically (dependency resolution, optional crates)
- **Limits & Colimits** — products, coproducts, equalizers, coequalizers, pullbacks, pushouts
- **Kan Extensions** — the most general universal constructions in category theory
- **Yoneda Lemma** — the fundamental representation theorem
- **Sheaves on Categories** — presheaves, sheafification, stalks, global sections
- **Composition Engine** — automated functor composition pipeline for the ecosystem

This is "category theory, but make it compile."

---

## Key Idea

> "A category is a world. A functor is a translation between worlds. A natural transformation is a systematic comparison of translations."

The 320+ crates naturally organize into **subcategories** by domain:

```
Category(Analysis) ←—— Functor ——→ Category(Geometry)
       ↓                               ↓
  Natural Transform              Natural Transform
       ↓                               ↓
Category(Algebra) ←—— Functor ——→ Category(Topology)
```

Every arrow in this diagram is computable. The framework verifies category axioms (identity, associativity) at the level of actual Rust code and crate dependencies.

---

## Install

```toml
[dependencies]
lau-functor-network = "0.1.0"
```

Requires **Rust 2021 edition**. Dependencies: `serde`, `nalgebra`.

---

## Quick Start

```rust
use lau_functor_network::prelude::*;

// Build a category of crates
let mut cat = Category::new("Ecosystem");
cat.add_object(Object::new("lau-core", "LAU Core"));
cat.add_object(Object::new("lau-spectral", "Spectral Theory"));
cat.add_object(Object::new("lau-sheaf", "Sheaf Theory"));

cat.add_morphism(Morphism::new("dep-1",
    ObjectId("lau-spectral".into()),
    ObjectId("lau-core".into()),
).with_label("depends-on").with_weight(1.0));

// Verify category axioms
assert!(cat.check_left_identity(&some_morphism));   // id ∘ f = f
assert!(cat.check_right_identity(&some_morphism));  // f ∘ id = f
assert!(Category::check_associativity(&f, &g, &h)); // (f∘g)∘h = f∘(g∘h)

// Build the adjacency matrix (via nalgebra)
let (matrix, ids) = cat.adjacency_matrix();

// Create a functor between two categories
let functor = Functor::new("inclusion", source_cat, target_cat);

// Check naturality squares
let nt = NaturalTransformation::new("eta", functor_f, functor_g);
assert!(nt.check_naturality(&morph_a_to_b));
```

---

## API Reference

### `category` — Categories, Objects, Morphisms

| Type | Description |
|------|-------------|
| `ObjectId` | Unique identifier for a crate (e.g., `"lau-core"`) |
| `MorphismId` | Unique identifier for a dependency/connection |
| `Object` | A crate with id, name, version, and metadata |
| `Morphism` | A dependency: source → target with label and weight |
| `Category` | Collection of objects and morphisms with identity & composition |

**Key methods on `Category`:**

```rust
cat.add_object(obj);                    // Add a crate
cat.add_morphism(morphism)?;            // Add a dependency
cat.identity(&object_id);               // Get id: X → X
cat.hom(&src, &tgt);                    // All morphisms src → tgt
cat.morphisms_from(&src);               // Outgoing dependencies
cat.morphisms_to(&tgt);                 // Incoming dependencies
cat.check_left_identity(&m);            // id ∘ f = f
cat.check_right_identity(&m);           // f ∘ id = f
cat.adjacency_matrix();                 // nalgebra DMatrix<f64>
cat.is_subcategory_of(&other);          // Subcategory check
```

### `functor` — Functors

Structure-preserving maps between categories. A functor `F: C → D` maps objects to objects and morphisms to morphisms, preserving identity and composition.

```rust
let f = Functor::new("embedding", cat_c, cat_d);
f.map_object(&obj);       // F(X) in D
f.map_morphism(&mor);     // F(f) in D
f.check_preserves_identity();
f.check_preserves_composition(&m1, &m2);
```

### `natural_transform` — Natural Transformations

A family of morphisms `η_X: F(X) → G(X)` for each object X, such that for every `f: X → Y`, `G(f) ∘ η_X = η_Y ∘ F(f)` (naturality square).

```rust
let nt = NaturalTransformation::new("projection", functor_f, functor_g);
nt.component(&obj);                    // η_X
nt.check_naturality(&morph_x_to_y);    // Verifies the square commutes
```

### `adjunction` — Adjunctions

A pair of functors `F ⊣ G` where `Hom_D(FX, Y) ≅ Hom_C(X, GY)`. Includes unit `η: Id → GF` and counit `ε: FG → Id`.

```rust
let adj = Adjunction::new(functor_f, functor_g, unit, counit);
adj.check_triangle_identities();
```

### `monad` — Monads

A monad `(T, η, μ)` on a category: an endofunctor `T` with unit `η: Id → T` and multiplication `μ: T² → T`.

```rust
let monad = Monad::new(endofunctor, unit, multiply);
monad.check_associativity();   // μ ∘ Tμ = μ ∘ μT
monad.check_unit_left();       // μ ∘ ηT = id
monad.check_unit_right();      // μ ∘ Tη = id
```

### `limit_colimit` — Limits and Colimits

Universal constructions: products, coproducts, equalizers, coequalizers, pullbacks, pushouts.

```rust
let product = LimitColimit::product(&cat, &obj_a, &obj_b);      // A × B
let coproduct = LimitColimit::coproduct(&cat, &obj_a, &obj_b);  // A ⊔ B
let pullback = LimitColimit::pullback(&cat, &f, &g);            // Pullback
let equalizer = LimitColimit::equalizer(&cat, &f, &g);          // Eq(f, g)
```

### `yoneda` — Yoneda Lemma

The Yoneda embedding `Y: C → Set^{C^op}` and the Yoneda lemma: `Nat(Hom(-, X), F) ≅ F(X)`.

```rust
let embedding = Yoneda::embed(&cat);
let result = Yoneda::lemma(&functor_f, &object_x); // ≅ F(X)
```

### `kan_extension` — Kan Extensions

Left and right Kan extensions: the most general universal constructions, of which limits, colimits, and adjunctions are special cases.

```rust
let lan = KanExtension::left(&functor_f, &functor_g);   // Lan_F G
let ran = KanExtension::right(&functor_f, &functor_g);  // Ran_F G
```

### `sheaf_on_category` — Sheaves

Presheaves (contravariant functors `C^op → Set`), sheafification, stalks, and global sections.

```rust
let mut presheaf = Presheaf::new("ecosystem_sections");
presheaf.assign_sections(object_id, vec!["s1", "s2"]);
presheaf.add_restriction(src, tgt, "restrict_map");

let sheaf = Sheaf::from_presheaf(presheaf);
sheaf.is_sheaf();                    // Checks gluing axioms
sheaf.stalk(&object_id);             // Direct limit of sections
let globals = global_sections(&sheaf); // Sections over the whole category
```

### `composition_engine` — Automated Composition

Pipeline that automatically discovers and composes functors across the ecosystem to build complex cross-domain mappings.

```rust
let engine = CompositionEngine::new(&ecosystem_category);
let path = engine.find_composition(&source, &target);
let result = engine.compose_along(path);
```

---

## How It Works

### Architecture

```
┌──────────────────────────────────────────────────┐
│                 Category (C)                      │
│  Objects = Crates, Morphisms = Dependencies       │
│  Identity: id_X for each crate X                  │
│  Composition: f;g when cod(f) = dom(g)            │
├──────────────────────────────────────────────────┤
│  Functor (F: C → D)    │  Natural Transform (η)  │
│  Maps objects → objects │  Maps functors → functors│
│  Maps morphs → morphs   │  Naturality squares      │
├──────────────────────────────────────────────────┤
│  Adjunction (F ⊣ G)    │  Monad (T, η, μ)        │
│  Unit + Counit          │  Unit + Multiply         │
│  Triangle identities    │  Associativity           │
├──────────────────────────────────────────────────┤
│  Limits / Colimits     │  Kan Extensions           │
│  Products, Coproducts  │  Left & Right Kan         │
│  Equalizers, Pullbacks │  (generalize limits)      │
├──────────────────────────────────────────────────┤
│  Yoneda Lemma          │  Sheaves on Categories    │
│  Nat(Hom(-,X), F) ≅ F(X)│ Presheaf → Sheaf        │
│                        │  Stalks, Global Sections  │
├──────────────────────────────────────────────────┤
│           Composition Engine                       │
│  Automated functor composition across ecosystem    │
└──────────────────────────────────────────────────┘
```

### Morphism Composition

Two morphisms compose when the target of the first equals the source of the second:

```
f: A → B,  g: B → C   ⟹   g ∘ f: A → C
```

Weights multiply: `weight(g ∘ f) = weight(f) × weight(g)`.

The framework verifies:
- **Left identity**: `id ∘ f = f`
- **Right identity**: `f ∘ id = f`
- **Associativity**: `(f ∘ g) ∘ h = f ∘ (g ∘ h)`

### Adjunctions as "Free/Forgetful" Pairs

The most important adjunction in the ecosystem is `Free ⊣ Forgetful`:
- **Free functor**: Given a set of features, build the "free" crate that has exactly those features and all their consequences
- **Forgetful functor**: Given a crate, forget its structure and just return its set of features

The adjunction says: "maps from Free(S) to a crate C correspond naturally to maps from S to Forget(C)."

---

## The Math

### Categories

A category `C` consists of:
- A collection of **objects** `Ob(C)`
- For each pair `X, Y ∈ Ob(C)`, a set `Hom(X, Y)` of **morphisms**
- For each `X`, an identity `id_X ∈ Hom(X, X)`
- Composition `∘: Hom(Y, Z) × Hom(X, Y) → Hom(X, Z)` satisfying associativity and identity laws

### Functors

A functor `F: C → D` assigns:
- To each object `X ∈ C`, an object `F(X) ∈ D`
- To each morphism `f: X → Y` in `C`, a morphism `F(f): F(X) → F(Y)` in `D`
- Preserving: `F(id_X) = id_{F(X)}` and `F(g ∘ f) = F(g) ∘ F(f)`

### Yoneda Lemma

For any functor `F: C → Set` and any object `X ∈ C`:

$$\text{Nat}(\text{Hom}_C(-, X), F) \cong F(X)$$

This is arguably the most important result in category theory. It says that an object is completely determined by its relationships to all other objects.

### Kan Extensions

Given functors `F: C → D` and `G: C → E`, the **left Kan extension** `Lan_F G: D → E` is the "best approximation" of `G` along `F`. Formally, it's the left adjoint to the precomposition functor `F*`.

All limits, colimits, and adjunctions are special cases of Kan extensions.

### Sheaves

A **presheaf** on `C` is a contravariant functor `F: C^op → Set`. A **sheaf** is a presheaf satisfying gluing conditions: if local sections agree on overlaps, they patch together into a unique global section.

The **stalk** at an object `X` is the direct limit `colim_{U ⊇ X} F(U)` — the "germ" of sections near `X`.

---

## Test Coverage

**102 tests** covering:

- Object and morphism creation, composition, and failure cases (24 tests)
- Category axioms: identity, associativity, subcategory (18 tests)
- Functor preservation of identity and composition (10 tests)
- Natural transformation naturality squares (8 tests)
- Adjunction triangle identities (6 tests)
- Monad laws: associativity, unit (8 tests)
- Limits and colimits: products, coproducts, pullbacks, pushouts (10 tests)
- Yoneda lemma verification (6 tests)
- Kan extension universal properties (6 tests)
- Sheaf axioms: presheaf, sheafification, stalks, global sections (6 tests)

---

## License

MIT
