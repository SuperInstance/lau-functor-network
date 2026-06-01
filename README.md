# lau-functor-network

Category-theoretic framework for composing the 320+ crate SuperInstance ecosystem.

Each crate in the SuperInstance ecosystem is an object in a category. Morphisms are the mathematical connections between them. Functors map between sub-categories.

## Modules

| Module | Description |
|--------|-------------|
| `category` | Objects (crates), morphisms (dependencies), identity, composition, associativity |
| `functor` | Structure-preserving maps between categories (forgetful, free, faithful, full) |
| `natural_transform` | Maps between functors connecting different views of the same math |
| `adjunction` | Pairs of functors (F ⊣ G) — obs/control, free/forgetful, discrete/continuous |
| `limit_colimit` | Products, coproducts, pullbacks, pushouts, equalizers |
| `monad` | Computational effects: Maybe, State, Reader, Writer monads |
| `yoneda` | Yoneda lemma: objects are determined by their relationships |
| `kan_extension` | Universal construction extending functors (left and right Kan extensions) |
| `sheaf_on_category` | Sheaves on the crate dependency graph with gluing conditions |
| `composition_engine` | Automatically compose crate functionality via category theory |

## Dependencies

- `serde` — serialization for all category-theoretic structures
- `nalgebra` — adjacency matrices and linear algebra on morphism weights

## Usage

```rust
use lau_functor_network::prelude::*;

// Create a category for your crate ecosystem
let mut cat = Category::new("SuperInstance");
cat.add_object(Object::new("lau-core", "Core Foundation"));
cat.add_object(Object::new("lau-agents", "Agent Framework"));

// Add dependency morphism
cat.add_morphism(Morphism::new("dep", ObjectId("lau-agents".into()), ObjectId("lau-core".into()))).unwrap();

// Use the composition engine
let mut engine = CompositionEngine::new(cat);
let deps = engine.dependencies(&ObjectId("lau-agents".into()));
```

## Test Coverage

102+ tests covering all modules.
