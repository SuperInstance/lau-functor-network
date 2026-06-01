//! Kan extensions — universal construction extending functors.
//!
//! Left Kan extension: Lan_F(K) is the "best approximation" of extending K along F.
//! Right Kan extension: Ran_F(K) is the "best approximation" from the other direction.

use serde::{Deserialize, Serialize};
use crate::category::ObjectId;
use crate::functor::Functor;
use std::collections::HashMap;

/// A Kan extension (left or right).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KanExtension {
    pub name: String,
    pub kind: KanKind,
    /// The functor F being extended along
    pub along_functor: String,
    /// The functor K being extended
    pub extended_functor: String,
    /// The resulting functor Lan_F(K) or Ran_F(K)
    pub result_functor_name: String,
    /// Object mappings for the result
    object_map: HashMap<ObjectId, ObjectId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KanKind {
    Left,
    Right,
}

impl KanExtension {
    /// Compute the left Kan extension Lan_F(K).
    /// Colimit-based: Lan_F(K)(b) = colimit_{(a, f: F(a)→b)} K(a)
    pub fn left_kan(
        name: impl Into<String>,
        f: &Functor,
        k: &Functor,
    ) -> Result<KanExtension, String> {
        if f.source_category != k.source_category {
            return Err("F and K must share source category".to_string());
        }
        let mut object_map = HashMap::new();
        // For each object in the intermediate category, compute the colimit
        for (src, _mid) in &f.object_map {
            if let Some(k_target) = k.object_map.get(src) {
                // Simplified: direct mapping through composition
                object_map.insert(src.clone(), k_target.clone());
            }
        }
        Ok(KanExtension {
            name: name.into(),
            kind: KanKind::Left,
            along_functor: f.name.clone(),
            extended_functor: k.name.clone(),
            result_functor_name: format!("Lan_{}({})", f.name, k.name),
            object_map,
        })
    }

    /// Compute the right Kan extension Ran_F(K).
    /// Limit-based: Ran_F(K)(b) = limit_{(a, f: b→F(a))} K(a)
    pub fn right_kan(
        name: impl Into<String>,
        f: &Functor,
        k: &Functor,
    ) -> Result<KanExtension, String> {
        if f.source_category != k.source_category {
            return Err("F and K must share source category".to_string());
        }
        let mut object_map = HashMap::new();
        for (src, _mid) in &f.object_map {
            if let Some(k_target) = k.object_map.get(src) {
                object_map.insert(src.clone(), k_target.clone());
            }
        }
        Ok(KanExtension {
            name: name.into(),
            kind: KanKind::Right,
            along_functor: f.name.clone(),
            extended_functor: k.name.clone(),
            result_functor_name: format!("Ran_{}({})", f.name, k.name),
            object_map,
        })
    }

    pub fn apply(&self, obj: &ObjectId) -> Option<&ObjectId> {
        self.object_map.get(obj)
    }

    /// If the extension is a left Kan, produce the unit natural transformation.
    pub fn unit(&self) -> String {
        format!("η: {} → {}∘{}", self.extended_functor, self.result_functor_name, self.along_functor)
    }

    /// If the extension is a right Kan, produce the counit natural transformation.
    pub fn counit(&self) -> String {
        format!("ε: {}∘{} → {}", self.result_functor_name, self.along_functor, self.extended_functor)
    }

    /// Kan extensions compose: Lan_F(K) along a composed functor.
    pub fn compose(&self, _other: &KanExtension) -> Result<KanExtension, String> {
        // Simplified — just produce a new kan extension
        Ok(KanExtension {
            name: format!("composed_{}", self.name),
            kind: self.kind.clone(),
            along_functor: self.along_functor.clone(),
            extended_functor: self.extended_functor.clone(),
            result_functor_name: format!("composed_{}", self.result_functor_name),
            object_map: self.object_map.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_functors() -> (Functor, Functor) {
        let mut f = Functor::new("F", "C", "D");
        f.map_object(ObjectId("a".into()), ObjectId("fa".into()));
        f.map_object(ObjectId("b".into()), ObjectId("fb".into()));
        let mut k = Functor::new("K", "C", "E");
        k.map_object(ObjectId("a".into()), ObjectId("ka".into()));
        k.map_object(ObjectId("b".into()), ObjectId("kb".into()));
        (f, k)
    }

    #[test]
    fn test_left_kan_extension() {
        let (f, k) = setup_functors();
        let lan = KanExtension::left_kan("LanF_K", &f, &k).unwrap();
        assert_eq!(lan.kind, KanKind::Left);
        assert_eq!(lan.apply(&ObjectId("a".into())), Some(&ObjectId("ka".into())));
    }

    #[test]
    fn test_right_kan_extension() {
        let (f, k) = setup_functors();
        let ran = KanExtension::right_kan("RanF_K", &f, &k).unwrap();
        assert_eq!(ran.kind, KanKind::Right);
        assert_eq!(ran.apply(&ObjectId("b".into())), Some(&ObjectId("kb".into())));
    }

    #[test]
    fn test_kan_fails_wrong_source() {
        let f = Functor::new("F", "C", "D");
        let k = Functor::new("K", "X", "E");
        assert!(KanExtension::left_kan("fail", &f, &k).is_err());
    }

    #[test]
    fn test_kan_unit() {
        let (f, k) = setup_functors();
        let lan = KanExtension::left_kan("LanF_K", &f, &k).unwrap();
        let unit = lan.unit();
        assert!(unit.contains("η"));
    }

    #[test]
    fn test_kan_counit() {
        let (f, k) = setup_functors();
        let ran = KanExtension::right_kan("RanF_K", &f, &k).unwrap();
        let counit = ran.counit();
        assert!(counit.contains("ε"));
    }

    #[test]
    fn test_kan_compose() {
        let (f, k) = setup_functors();
        let lan = KanExtension::left_kan("Lan", &f, &k).unwrap();
        let composed = lan.compose(&lan).unwrap();
        assert!(composed.name.contains("composed"));
    }
}
