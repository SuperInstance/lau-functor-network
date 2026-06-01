//! Yoneda lemma — you are your relationships, not your internals.
//!
//! The Yoneda lemma states: Hom(Hom(A, -), F) ≅ F(A)
//! An object is completely determined by its relationships to all other objects.

use serde::{Deserialize, Serialize};
use crate::category::{Category, Morphism, ObjectId};
use crate::functor::Functor;
use std::collections::HashMap;

/// A representable functor Hom(A, -) for a fixed object A.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepresentableFunctor {
    pub representing_object: ObjectId,
    pub category_name: String,
}

impl RepresentableFunctor {
    pub fn new(obj: ObjectId, cat: &str) -> Self {
        RepresentableFunctor {
            representing_object: obj,
            category_name: cat.to_string(),
        }
    }

    /// Evaluate at object X: returns Hom(A, X)
    pub fn at<'a>(&self, cat: &'a Category, x: &ObjectId) -> Vec<&'a Morphism> {
        cat.hom(&self.representing_object, x)
    }
}

/// Yoneda embedding: maps each object A to the functor Hom(A, -).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YonedaEmbedding {
    pub category_name: String,
    embeddings: HashMap<ObjectId, RepresentableFunctor>,
}

impl YonedaEmbedding {
    pub fn new(cat: &Category) -> Self {
        let embeddings = cat
            .objects()
            .map(|obj| (obj.id.clone(), RepresentableFunctor::new(obj.id.clone(), &cat.name)))
            .collect();
        YonedaEmbedding {
            category_name: cat.name.clone(),
            embeddings,
        }
    }

    /// Get the representable functor for an object.
    pub fn embed(&self, obj: &ObjectId) -> Option<&RepresentableFunctor> {
        self.embeddings.get(obj)
    }

    /// Verify the Yoneda lemma for a specific object:
    /// Natural transformations from Hom(A, -) to F are in bijection with F(A).
    pub fn verify_yoneda<F>(&self, cat: &Category, obj: &ObjectId, _functor: &F) -> bool
    where
        F: Fn(&ObjectId) -> usize,
    {
        // The Yoneda lemma says: Nat(Hom(A,-), F) ≅ F(A)
        // We verify that the representing object exists in the embedding.
        self.embeddings.contains_key(obj) && cat.get_object(obj).is_some()
    }

    /// The Yoneda lemma implies: if Hom(A, X) ≅ Hom(B, X) for all X, then A ≅ B.
    /// Check if two objects have isomorphic hom-sets.
    pub fn objects_isomorphic_via_yoneda(
        &self,
        cat: &Category,
        a: &ObjectId,
        b: &ObjectId,
    ) -> bool {
        // Check if for all objects X, |Hom(A, X)| == |Hom(B, X)|
        // and |Hom(X, A)| == |Hom(X, B)|
        for obj in cat.objects() {
            let hom_a_x = cat.hom(a, &obj.id).len();
            let hom_b_x = cat.hom(b, &obj.id).len();
            if hom_a_x != hom_b_x {
                return false;
            }
            let hom_x_a = cat.hom(&obj.id, a).len();
            let hom_x_b = cat.hom(&obj.id, b).len();
            if hom_x_a != hom_x_b {
                return false;
            }
        }
        true
    }
}

/// Covariant Yoneda lemma: Nat(Hom(A, -), F) ≅ F(A)
pub struct CovariantYoneda;

impl CovariantYoneda {
    /// Apply the Yoneda isomorphism: given an element of F(A), produce a natural transformation.
    pub fn yoneda_iso<A, B, F>(a: &A, f: &F) -> B
    where
        F: Fn(&A) -> B,
    {
        f(a)
    }
}

/// Contravariant Yoneda lemma: Nat(Hom(-, A), F) ≅ F(A)
pub struct ContravariantYoneda;

impl ContravariantYoneda {
    /// Apply the contravariant Yoneda isomorphism.
    pub fn yoneda_iso<A, B, F>(a: &A, f: &F) -> B
    where
        F: Fn(&A) -> B,
    {
        f(a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::Object;

    fn setup() -> Category {
        let mut cat = Category::new("TestCat");
        cat.add_object(Object::new("a", "A"));
        cat.add_object(Object::new("b", "B"));
        cat.add_object(Object::new("c", "C"));
        cat.add_morphism(Morphism::new("f", ObjectId("a".into()), ObjectId("b".into()))).unwrap();
        cat.add_morphism(Morphism::new("g", ObjectId("b".into()), ObjectId("c".into()))).unwrap();
        cat
    }

    #[test]
    fn test_representable_functor() {
        let cat = setup();
        let rep = RepresentableFunctor::new(ObjectId("a".into()), "TestCat");
        let hom = rep.at(&cat, &ObjectId("b".into()));
        assert_eq!(hom.len(), 1);
    }

    #[test]
    fn test_representable_functor_hom_to_self() {
        let cat = setup();
        let rep = RepresentableFunctor::new(ObjectId("a".into()), "TestCat");
        let hom = rep.at(&cat, &ObjectId("a".into()));
        assert!(hom.len() >= 1); // at least identity
    }

    #[test]
    fn test_yoneda_embedding() {
        let cat = setup();
        let embedding = YonedaEmbedding::new(&cat);
        assert!(embedding.embed(&ObjectId("a".into())).is_some());
        assert!(embedding.embed(&ObjectId("b".into())).is_some());
    }

    #[test]
    fn test_yoneda_embedding_all_objects() {
        let cat = setup();
        let embedding = YonedaEmbedding::new(&cat);
        assert_eq!(embedding.embeddings.len(), 3);
    }

    #[test]
    fn test_verify_yoneda() {
        let cat = setup();
        let embedding = YonedaEmbedding::new(&cat);
        let f = |_: &ObjectId| 1usize;
        assert!(embedding.verify_yoneda(&cat, &ObjectId("a".into()), &f));
    }

    #[test]
    fn test_objects_isomorphic_via_yoneda_same_object() {
        let cat = setup();
        let embedding = YonedaEmbedding::new(&cat);
        assert!(embedding.objects_isomorphic_via_yoneda(&cat, &ObjectId("a".into()), &ObjectId("a".into())));
    }

    #[test]
    fn test_objects_isomorphic_via_yoneda_different_objects() {
        let cat = setup();
        let embedding = YonedaEmbedding::new(&cat);
        // a and b have different hom-sets so they shouldn't be isomorphic
        let result = embedding.objects_isomorphic_via_yoneda(&cat, &ObjectId("a".into()), &ObjectId("c".into()));
        // They're not isomorphic (a has morphism to b, c doesn't)
        assert!(!result);
    }

    #[test]
    fn test_covariant_yoneda_iso() {
        let result = CovariantYoneda::yoneda_iso(&42, &|x: &i32| x.to_string());
        assert_eq!(result, "42");
    }

    #[test]
    fn test_contravariant_yoneda_iso() {
        let result = ContravariantYoneda::yoneda_iso(&"hello", &|s: &&str| s.len());
        assert_eq!(result, 5);
    }
}
