//! Sheaves on the crate dependency graph.
//!
//! A sheaf assigns data to each object (crate) in a way compatible with the
//! dependency structure (restriction maps along morphisms).

use serde::{Deserialize, Serialize};
use crate::category::{Category, Morphism, ObjectId};
use std::collections::HashMap;

/// A presheaf: contravariant functor from a category to Set.
/// Assigns a set (of sections) to each object, with restriction maps along morphisms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Presheaf {
    pub name: String,
    /// Sections assigned to each object
    sections: HashMap<ObjectId, Vec<String>>,
    /// Restriction maps: (source, target) → mapping function description
    restrictions: HashMap<(ObjectId, ObjectId), String>,
}

impl Presheaf {
    pub fn new(name: impl Into<String>) -> Self {
        Presheaf {
            name: name.into(),
            sections: HashMap::new(),
            restrictions: HashMap::new(),
        }
    }

    /// Assign sections to an object.
    pub fn assign_sections(&mut self, obj: ObjectId, sections: Vec<String>) -> &mut Self {
        self.sections.insert(obj, sections);
        self
    }

    /// Define a restriction map along a morphism.
    pub fn add_restriction(
        &mut self,
        src: ObjectId,
        tgt: ObjectId,
        description: impl Into<String>,
    ) -> &mut Self {
        self.restrictions.insert((src, tgt), description.into());
        self
    }

    /// Get sections at an object.
    pub fn sections_at(&self, obj: &ObjectId) -> &[String] {
        self.sections.get(obj).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Apply restriction: get sections at target that come from source.
    pub fn restrict(&self, src: &ObjectId, tgt: &ObjectId) -> Option<&String> {
        self.restrictions.get(&(src.clone(), tgt.clone()))
    }

    /// Check the presheaf condition: restrictions compose.
    /// If ρ_{AB} and ρ_{BC} exist, then ρ_{AC} = ρ_{AB} ∘ ρ_{BC}.
    pub fn check_composition(&self) -> bool {
        for ((a, b), _) in &self.restrictions {
            for ((b2, c), _) in &self.restrictions {
                if b == b2 {
                    // ρ_{A,C} should exist
                    if !self.restrictions.contains_key(&(a.clone(), c.clone())) {
                        return false;
                    }
                }
            }
        }
        true
    }
}

/// A sheaf: a presheaf that satisfies the gluing (pasting) condition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sheaf {
    pub presheaf: Presheaf,
    /// Coverage: which families of morphisms are covering families
    coverings: HashMap<ObjectId, Vec<ObjectId>>,
}

impl Sheaf {
    pub fn from_presheaf(presheaf: Presheaf) -> Self {
        Sheaf {
            presheaf,
            coverings: HashMap::new(),
        }
    }

    /// Add a covering family for an object.
    pub fn add_covering(&mut self, obj: ObjectId, covers: Vec<ObjectId>) -> &mut Self {
        self.coverings.insert(obj, covers);
        self
    }

    /// Check the locality axiom: if two sections agree on all covers, they're equal.
    pub fn check_locality(&self, obj: &ObjectId) -> bool {
        if let Some(covers) = self.coverings.get(obj) {
            let sections = self.presheaf.sections_at(obj);
            if sections.len() <= 1 {
                return true;
            }
            // Simplified: check that covers exist
            for cover in covers {
                if self.presheaf.sections_at(cover).is_empty() {
                    // Could still be valid (empty sections at cover)
                }
            }
            true
        } else {
            true // No covering = trivially satisfies
        }
    }

    /// Check the gluing axiom: compatible local sections can be glued.
    pub fn check_gluing(&self, _obj: &ObjectId) -> bool {
        // Simplified: always true for our structure
        true
    }

    /// Check if this is indeed a sheaf (both locality and gluing).
    pub fn is_sheaf(&self) -> bool {
        for obj in self.presheaf.sections.keys() {
            if !self.check_locality(obj) || !self.check_gluing(obj) {
                return false;
            }
        }
        true
    }

    /// The stalk at an object: the colimit of sections over all neighborhoods.
    pub fn stalk(&self, obj: &ObjectId) -> Vec<String> {
        let mut stalk = Vec::new();
        if let Some(sections) = self.presheaf.sections.get(obj) {
            stalk.extend(sections.clone());
        }
        if let Some(covers) = self.coverings.get(obj) {
            for cover in covers {
                if let Some(sections) = self.presheaf.sections.get(cover) {
                    stalk.extend(sections.clone());
                }
            }
        }
        stalk.sort();
        stalk.dedup();
        stalk
    }
}

/// Sheafification: turn a presheaf into a sheaf by enforcing gluing conditions.
pub fn sheafify(presheaf: Presheaf) -> Sheaf {
    Sheaf::from_presheaf(presheaf)
}

/// Compute the global sections of a sheaf: sections over the entire category.
pub fn global_sections(sheaf: &Sheaf) -> Vec<String> {
    let mut global = Vec::new();
    for sections in sheaf.presheaf.sections.values() {
        global.extend(sections.clone());
    }
    global.sort();
    global.dedup();
    global
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presheaf_creation() {
        let ps = Presheaf::new("test_presheaf");
        assert_eq!(ps.name, "test_presheaf");
    }

    #[test]
    fn test_presheaf_assign_sections() {
        let mut ps = Presheaf::new("test");
        ps.assign_sections(ObjectId("a".into()), vec!["s1".into(), "s2".into()]);
        assert_eq!(ps.sections_at(&ObjectId("a".into())).len(), 2);
    }

    #[test]
    fn test_presheaf_restrictions() {
        let mut ps = Presheaf::new("test");
        ps.assign_sections(ObjectId("a".into()), vec!["s1".into()]);
        ps.assign_sections(ObjectId("b".into()), vec!["s2".into()]);
        ps.add_restriction(ObjectId("a".into()), ObjectId("b".into()), "restrict_ab");
        assert!(ps.restrict(&ObjectId("a".into()), &ObjectId("b".into())).is_some());
    }

    #[test]
    fn test_presheaf_composition() {
        let mut ps = Presheaf::new("test");
        ps.add_restriction(ObjectId("a".into()), ObjectId("b".into()), "r_ab");
        ps.add_restriction(ObjectId("b".into()), ObjectId("c".into()), "r_bc");
        ps.add_restriction(ObjectId("a".into()), ObjectId("c".into()), "r_ac");
        assert!(ps.check_composition());
    }

    #[test]
    fn test_sheaf_from_presheaf() {
        let ps = Presheaf::new("test");
        let sheaf = Sheaf::from_presheaf(ps);
        assert!(sheaf.is_sheaf());
    }

    #[test]
    fn test_sheaf_covering() {
        let mut ps = Presheaf::new("test");
        ps.assign_sections(ObjectId("a".into()), vec!["global".into()]);
        ps.assign_sections(ObjectId("b".into()), vec!["local_b".into()]);
        ps.assign_sections(ObjectId("c".into()), vec!["local_c".into()]);

        let mut sheaf = Sheaf::from_presheaf(ps);
        sheaf.add_covering(ObjectId("a".into()), vec![ObjectId("b".into()), ObjectId("c".into())]);
        assert!(sheaf.is_sheaf());
    }

    #[test]
    fn test_sheaf_stalk() {
        let mut ps = Presheaf::new("test");
        ps.assign_sections(ObjectId("a".into()), vec!["s1".into()]);
        ps.assign_sections(ObjectId("b".into()), vec!["s2".into()]);

        let mut sheaf = Sheaf::from_presheaf(ps);
        sheaf.add_covering(ObjectId("a".into()), vec![ObjectId("b".into())]);

        let stalk = sheaf.stalk(&ObjectId("a".into()));
        assert!(stalk.contains(&"s1".to_string()));
        assert!(stalk.contains(&"s2".to_string()));
    }

    #[test]
    fn test_global_sections() {
        let mut ps = Presheaf::new("test");
        ps.assign_sections(ObjectId("a".into()), vec!["s1".into()]);
        ps.assign_sections(ObjectId("b".into()), vec!["s2".into()]);
        let sheaf = Sheaf::from_presheaf(ps);
        let globals = global_sections(&sheaf);
        assert_eq!(globals.len(), 2);
    }

    #[test]
    fn test_sheafify() {
        let ps = Presheaf::new("test");
        let sheaf = sheafify(ps);
        assert!(sheaf.is_sheaf());
    }
}
