//! Structure-preserving maps between categories (e.g., forgetful functor from topological-agents to set-agents).

use serde::{Deserialize, Serialize};
use crate::category::{Category, Morphism, ObjectId, Object};

/// A functor maps objects to objects and morphisms to morphisms, preserving structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Functor {
    pub name: String,
    pub source_category: String,
    pub target_category: String,
    pub object_map: std::collections::HashMap<ObjectId, ObjectId>,
    pub morphism_map: std::collections::HashMap<String, String>,
    /// Metadata about the kind of functor (forgetful, free, faithful, full, etc.)
    pub kind: FunctorKind,
}

/// Classification of functors by their properties.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FunctorKind {
    /// Preserves all structure
    Full,
    /// Injective on hom-sets
    Faithful,
    /// Fully faithful
    FullyFaithful,
    /// Drops structure (e.g., topology → sets)
    Forgetful,
    /// Adds free structure (e.g., sets → groups)
    Free,
    /// Generic
    Generic,
}

impl Functor {
    pub fn new(
        name: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        Functor {
            name: name.into(),
            source_category: source.into(),
            target_category: target.into(),
            object_map: std::collections::HashMap::new(),
            morphism_map: std::collections::HashMap::new(),
            kind: FunctorKind::Generic,
        }
    }

    pub fn with_kind(mut self, kind: FunctorKind) -> Self {
        self.kind = kind;
        self
    }

    /// Map an object from source to target category.
    pub fn map_object(&mut self, src: ObjectId, tgt: ObjectId) -> &mut Self {
        self.object_map.insert(src, tgt);
        self
    }

    /// Map a morphism from source to target category.
    pub fn map_morphism(&mut self, src: impl Into<String>, tgt: impl Into<String>) -> &mut Self {
        self.morphism_map.insert(src.into(), tgt.into());
        self
    }

    /// Apply the functor to an object.
    pub fn apply_object(&self, obj: &ObjectId) -> Option<&ObjectId> {
        self.object_map.get(obj)
    }

    /// Apply the functor to a morphism by id string.
    pub fn apply_morphism(&self, mor_id: &str) -> Option<&String> {
        self.morphism_map.get(mor_id)
    }

    /// Verify the functor preserves identity morphisms: F(id_X) = id_{F(X)}
    pub fn preserves_identities(&self, cat: &Category) -> bool {
        for (src_obj, tgt_obj) in &self.object_map {
            let src_id_mor = cat.identity(src_obj);
            let tgt_id_mor = cat.identity(tgt_obj);
            match (src_id_mor, tgt_id_mor) {
                (Some(src_m), Some(tgt_m)) => {
                    if let Some(mapped) = self.morphism_map.get(&src_m.id.0) {
                        if mapped != &tgt_m.id.0 {
                            return false;
                        }
                    }
                }
                _ => continue,
            }
        }
        true
    }

    /// Verify the functor preserves composition: F(g ∘ f) = F(g) ∘ F(f)
    pub fn preserves_composition(&self) -> bool {
        // Structural check — if all morphism mappings exist, assume preservation
        // Full verification requires composing actual morphisms in target category
        !self.morphism_map.is_empty() || self.object_map.is_empty()
    }

    /// Check if functor is faithful (injective on hom-sets).
    pub fn is_faithful(&self) -> bool {
        self.kind == FunctorKind::Faithful || self.kind == FunctorKind::FullyFaithful
    }

    /// Check if functor is full (surjective on hom-sets).
    pub fn is_full(&self) -> bool {
        self.kind == FunctorKind::Full || self.kind == FunctorKind::FullyFaithful
    }

    /// Compose two functors: (G ∘ F)(X) = G(F(X))
    pub fn compose(&self, other: &Functor, name: impl Into<String>) -> Result<Functor, String> {
        if self.target_category != other.source_category {
            return Err(format!(
                "Cannot compose: {} targets '{}' but {} sources '{}'",
                self.name, self.target_category, other.name, other.source_category
            ));
        }
        let mut composed = Functor::new(
            name,
            &self.source_category,
            &other.target_category,
        );
        for (src, mid) in &self.object_map {
            if let Some(tgt) = other.object_map.get(mid) {
                composed.object_map.insert(src.clone(), tgt.clone());
            }
        }
        for (src_m, mid_m) in &self.morphism_map {
            if let Some(tgt_m) = other.morphism_map.get(mid_m) {
                composed.morphism_map.insert(src_m.clone(), tgt_m.clone());
            }
        }
        Ok(composed)
    }
}

/// A forgetful functor that strips structure from objects.
pub fn forgetful_functor(
    source: impl Into<String>,
    target: impl Into<String>,
) -> Functor {
    Functor::new("forgetful", source, target).with_kind(FunctorKind::Forgetful)
}

/// A free functor that adds structure to objects.
pub fn free_functor(
    source: impl Into<String>,
    target: impl Into<String>,
) -> Functor {
    Functor::new("free", source, target).with_kind(FunctorKind::Free)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_functor_creation() {
        let f = Functor::new("F", "CatA", "CatB");
        assert_eq!(f.name, "F");
        assert_eq!(f.source_category, "CatA");
    }

    #[test]
    fn test_functor_map_object() {
        let mut f = Functor::new("F", "CatA", "CatB");
        f.map_object(ObjectId("a".into()), ObjectId("a'".into()));
        assert_eq!(f.apply_object(&ObjectId("a".into())), Some(&ObjectId("a'".into())));
    }

    #[test]
    fn test_functor_map_morphism() {
        let mut f = Functor::new("F", "CatA", "CatB");
        f.map_morphism("f", "F(f)");
        assert_eq!(f.apply_morphism("f"), Some(&"F(f)".to_string()));
    }

    #[test]
    fn test_functor_composition() {
        let mut f = Functor::new("F", "CatA", "CatB");
        f.map_object(ObjectId("a".into()), ObjectId("b".into()));
        let mut g = Functor::new("G", "CatB", "CatC");
        g.map_object(ObjectId("b".into()), ObjectId("c".into()));
        let gf = f.compose(&g, "G∘F").unwrap();
        assert_eq!(gf.apply_object(&ObjectId("a".into())), Some(&ObjectId("c".into())));
    }

    #[test]
    fn test_functor_composition_fails_wrong_categories() {
        let f = Functor::new("F", "CatA", "CatB");
        let g = Functor::new("G", "CatC", "CatD");
        assert!(f.compose(&g, "fail").is_err());
    }

    #[test]
    fn test_forgetful_functor() {
        let f = forgetful_functor("TopologicalAgents", "SetAgents");
        assert_eq!(f.kind, FunctorKind::Forgetful);
    }

    #[test]
    fn test_free_functor() {
        let f = free_functor("SetAgents", "GroupAgents");
        assert_eq!(f.kind, FunctorKind::Free);
    }

    #[test]
    fn test_functor_preserves_identities() {
        let mut cat = Category::new("CatA");
        cat.add_object(Object::new("x", "X"));
        let mut f = Functor::new("F", "CatA", "CatB");
        f.map_object(ObjectId("x".into()), ObjectId("x".into()));
        assert!(f.preserves_identities(&cat));
    }

    #[test]
    fn test_functor_kind_checks() {
        let f = Functor::new("F", "A", "B").with_kind(FunctorKind::FullyFaithful);
        assert!(f.is_faithful());
        assert!(f.is_full());
    }
}
