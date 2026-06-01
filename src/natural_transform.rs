//! Natural transformations — maps between functors connecting different views of the same math.

use serde::{Deserialize, Serialize};
use crate::category::ObjectId;
use crate::functor::Functor;

/// A component of a natural transformation at a specific object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NaturalTransformationComponent {
    pub object_id: ObjectId,
    pub morphism_label: String,
}

/// A natural transformation α: F ⇒ G between functors F, G: C → D.
/// For each object X in C, there is a morphism α_X: F(X) → G(X) in D,
/// such that α_Y ∘ F(f) = G(f) ∘ α_X for every f: X → Y.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NaturalTransformation {
    pub name: String,
    pub source_functor: String,
    pub target_functor: String,
    components: Vec<NaturalTransformationComponent>,
}

impl NaturalTransformation {
    pub fn new(
        name: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        NaturalTransformation {
            name: name.into(),
            source_functor: source.into(),
            target_functor: target.into(),
            components: Vec::new(),
        }
    }

    /// Add a component at object X: α_X: F(X) → G(X)
    pub fn add_component(&mut self, obj_id: ObjectId, label: impl Into<String>) -> &mut Self {
        self.components.push(NaturalTransformationComponent {
            object_id: obj_id,
            morphism_label: label.into(),
        });
        self
    }

    pub fn components(&self) -> &[NaturalTransformationComponent] {
        &self.components
    }

    pub fn component_at(&self, obj_id: &ObjectId) -> Option<&NaturalTransformationComponent> {
        self.components.iter().find(|c| &c.object_id == obj_id)
    }

    /// Vertical composition: given α: F ⇒ G and β: G ⇒ H, produce β ∘ α: F ⇒ H
    pub fn vertical_compose(
        &self,
        other: &NaturalTransformation,
        name: impl Into<String>,
    ) -> Result<NaturalTransformation, String> {
        if self.target_functor != other.source_functor {
            return Err(format!(
                "Cannot vertically compose: {} targets '{}', {} sources '{}'",
                self.name, self.target_functor, other.name, other.source_functor
            ));
        }
        let mut composed = NaturalTransformation::new(
            name,
            &self.source_functor,
            &other.target_functor,
        );
        for c in &self.components {
            if let Some(other_c) = other.component_at(&c.object_id) {
                composed.add_component(
                    c.object_id.clone(),
                    format!("{}∘{}", other_c.morphism_label, c.morphism_label),
                );
            }
        }
        Ok(composed)
    }

    /// Check naturality condition: for each object, the component exists.
    /// Full verification requires the target category's morphism composition.
    pub fn check_naturality(&self, functor_f: &Functor, functor_g: &Functor) -> bool {
        // Verify that for each component, both functors map the object
        for c in &self.components {
            if functor_f.apply_object(&c.object_id).is_none()
                || functor_g.apply_object(&c.object_id).is_none()
            {
                return false;
            }
        }
        true
    }
}

/// Identity natural transformation 1_F: F ⇒ F.
pub fn identity_natural_transformation(functor_name: &str) -> NaturalTransformation {
    NaturalTransformation::new(
        format!("id_{}", functor_name),
        functor_name,
        functor_name,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_natural_transformation_creation() {
        let nt = NaturalTransformation::new("alpha", "F", "G");
        assert_eq!(nt.name, "alpha");
        assert_eq!(nt.source_functor, "F");
    }

    #[test]
    fn test_add_components() {
        let mut nt = NaturalTransformation::new("alpha", "F", "G");
        nt.add_component(ObjectId("x".into()), "alpha_x");
        nt.add_component(ObjectId("y".into()), "alpha_y");
        assert_eq!(nt.components().len(), 2);
    }

    #[test]
    fn test_component_at() {
        let mut nt = NaturalTransformation::new("alpha", "F", "G");
        nt.add_component(ObjectId("x".into()), "alpha_x");
        let c = nt.component_at(&ObjectId("x".into())).unwrap();
        assert_eq!(c.morphism_label, "alpha_x");
    }

    #[test]
    fn test_vertical_composition() {
        let mut alpha = NaturalTransformation::new("alpha", "F", "G");
        alpha.add_component(ObjectId("x".into()), "a_x");
        let mut beta = NaturalTransformation::new("beta", "G", "H");
        beta.add_component(ObjectId("x".into()), "b_x");
        let composed = alpha.vertical_compose(&beta, "beta_alpha").unwrap();
        assert_eq!(composed.source_functor, "F");
        assert_eq!(composed.target_functor, "H");
        assert_eq!(composed.components().len(), 1);
    }

    #[test]
    fn test_vertical_composition_fails() {
        let alpha = NaturalTransformation::new("alpha", "F", "G");
        let beta = NaturalTransformation::new("beta", "H", "K");
        assert!(alpha.vertical_compose(&beta, "fail").is_err());
    }

    #[test]
    fn test_identity_natural_transformation() {
        let id = identity_natural_transformation("F");
        assert_eq!(id.source_functor, "F");
        assert_eq!(id.target_functor, "F");
    }

    #[test]
    fn test_naturality_check() {
        let mut nt = NaturalTransformation::new("alpha", "F", "G");
        nt.add_component(ObjectId("x".into()), "alpha_x");

        let mut f = crate::functor::Functor::new("F", "C", "D");
        f.map_object(ObjectId("x".into()), ObjectId("fx".into()));
        let mut g = crate::functor::Functor::new("G", "C", "D");
        g.map_object(ObjectId("x".into()), ObjectId("gx".into()));

        assert!(nt.check_naturality(&f, &g));
    }
}
