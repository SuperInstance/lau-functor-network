//! # lau-functor-network
//!
//! Category-theoretic framework for composing the 320+ crate SuperInstance ecosystem.
//!
//! Each crate is an object in a category. Morphisms are mathematical connections.
//! Functors map between sub-categories. The full power of category theory applied
//! to crate composition.

pub mod category;
pub mod functor;
pub mod natural_transform;
pub mod adjunction;
pub mod limit_colimit;
pub mod monad;
pub mod yoneda;
pub mod kan_extension;
pub mod sheaf_on_category;
pub mod composition_engine;

pub mod prelude {
    pub use crate::category::*;
    pub use crate::functor::*;
    pub use crate::natural_transform::*;
    pub use crate::adjunction::*;
    pub use crate::limit_colimit::*;
    pub use crate::monad::*;
    pub use crate::yoneda::*;
    pub use crate::kan_extension::*;
    pub use crate::sheaf_on_category::*;
    pub use crate::composition_engine::*;
}
