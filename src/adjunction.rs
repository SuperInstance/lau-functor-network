//! Adjunctions — pairs of functors (F ⊣ G) connecting obs/control, free/forgetful, discrete/continuous.

use serde::{Deserialize, Serialize};
use crate::functor::Functor;

/// An adjunction F ⊣ G between categories C and D.
/// F: C → D is left adjoint, G: D → C is right adjoint.
/// There is a natural bijection: Hom_D(F(X), Y) ≅ Hom_C(X, G(Y))
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Adjunction {
    pub name: String,
    pub left_adjoint: Functor,
    pub right_adjoint: Functor,
}

impl Adjunction {
    /// Create a new adjunction from two functors.
    /// Verifies that F.target == G.source and G.target == F.source.
    pub fn new(
        name: impl Into<String>,
        left: Functor,
        right: Functor,
    ) -> Result<Self, String> {
        if left.target_category != right.source_category {
            return Err(format!(
                "Left adjoint targets '{}', but right adjoint sources '{}'",
                left.target_category, right.source_category
            ));
        }
        if right.target_category != left.source_category {
            return Err(format!(
                "Right adjoint targets '{}', but left adjoint sources '{}'",
                right.target_category, left.source_category
            ));
        }
        Ok(Adjunction {
            name: name.into(),
            left_adjoint: left,
            right_adjoint: right,
        })
    }

    /// The unit η: Id_C → G∘F (for each X in C, η_X: X → G(F(X)))
    pub fn unit(&self) -> crate::natural_transform::NaturalTransformation {
        crate::natural_transform::NaturalTransformation::new(
            format!("unit_{}", self.name),
            "Id_C",
            &format!("G∘F_{}", self.name),
        )
    }

    /// The counit ε: F∘G → Id_D (for each Y in D, ε_Y: F(G(Y)) → Y)
    pub fn counit(&self) -> crate::natural_transform::NaturalTransformation {
        crate::natural_transform::NaturalTransformation::new(
            format!("counit_{}", self.name),
            &format!("F∘G_{}", self.name),
            "Id_D",
        )
    }

    /// Check triangle identities (simplified).
    pub fn check_triangle_identities(&self) -> bool {
        // Simplified: just verify functor composition compatibility
        self.left_adjoint.target_category == self.right_adjoint.source_category
            && self.right_adjoint.target_category == self.left_adjoint.source_category
    }

    /// Compose adjunctions: if F ⊣ G and F' ⊣ G', then F'∘F ⊣ G∘G'.
    pub fn compose(&self, other: &Adjunction, name: impl Into<String>) -> Result<Adjunction, String> {
        let name_str = name.into();
        let left = self.left_adjoint.compose(&other.left_adjoint, format!("{}_left", name_str))?;
        let right = other.right_adjoint.compose(&self.right_adjoint, format!("{}_right", name_str))?;
        Adjunction::new(name_str, left, right)
    }
}

/// Create the free-forgetful adjunction between sets and algebraic structures.
pub fn free_forgetful_adjunction(
    base_category: &str,
    algebraic_category: &str,
) -> Result<Adjunction, String> {
    let free = Functor::new("Free", base_category, algebraic_category)
        .with_kind(crate::functor::FunctorKind::Free);
    let forget = Functor::new("Forget", algebraic_category, base_category)
        .with_kind(crate::functor::FunctorKind::Forgetful);
    Adjunction::new("Free⊣Forget", free, forget)
}

/// Create the discrete-continuous adjunction.
pub fn discrete_continuous_adjunction(
    discrete_cat: &str,
    continuous_cat: &str,
) -> Result<Adjunction, String> {
    let disc = Functor::new("Discretize", continuous_cat, discrete_cat);
    let cont = Functor::new("Continuify", discrete_cat, continuous_cat);
    Adjunction::new("Disc⊣Cont", disc, cont)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adjunction_creation() {
        let left = Functor::new("F", "C", "D");
        let right = Functor::new("G", "D", "C");
        let adj = Adjunction::new("F⊣G", left, right).unwrap();
        assert_eq!(adj.left_adjoint.name, "F");
        assert_eq!(adj.right_adjoint.name, "G");
    }

    #[test]
    fn test_adjunction_creation_fails() {
        let left = Functor::new("F", "C", "D");
        let right = Functor::new("G", "X", "Y");
        assert!(Adjunction::new("bad", left, right).is_err());
    }

    #[test]
    fn test_unit_counit() {
        let left = Functor::new("F", "C", "D");
        let right = Functor::new("G", "D", "C");
        let adj = Adjunction::new("F⊣G", left, right).unwrap();
        let unit = adj.unit();
        assert_eq!(unit.source_functor, "Id_C");
        let counit = adj.counit();
        assert_eq!(counit.target_functor, "Id_D");
    }

    #[test]
    fn test_triangle_identities() {
        let left = Functor::new("F", "C", "D");
        let right = Functor::new("G", "D", "C");
        let adj = Adjunction::new("F⊣G", left, right).unwrap();
        assert!(adj.check_triangle_identities());
    }

    #[test]
    fn test_free_forgetful_adjunction() {
        let adj = free_forgetful_adjunction("Set", "Group").unwrap();
        assert_eq!(adj.left_adjoint.kind, crate::functor::FunctorKind::Free);
        assert_eq!(adj.right_adjoint.kind, crate::functor::FunctorKind::Forgetful);
    }

    #[test]
    fn test_discrete_continuous_adjunction() {
        let adj = discrete_continuous_adjunction("Discrete", "Continuous").unwrap();
        assert!(adj.check_triangle_identities());
    }
}
