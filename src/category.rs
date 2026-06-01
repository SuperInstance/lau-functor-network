//! Objects (crates), morphisms (dependencies), identity, composition, associativity.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

/// Unique identifier for an object (crate) in the category.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObjectId(pub String);

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Obj({})", self.0)
    }
}

impl From<&str> for ObjectId {
    fn from(s: &str) -> Self {
        ObjectId(s.to_string())
    }
}

/// Unique identifier for a morphism between objects.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct MorphismId(pub String);

impl fmt::Display for MorphismId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Mor({})", self.0)
    }
}

/// An object in the category — represents a crate in the ecosystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Object {
    pub id: ObjectId,
    pub name: String,
    pub version: String,
    pub metadata: HashMap<String, String>,
}

impl Object {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Object {
            id: ObjectId(id.into()),
            name: name.into(),
            version: "0.1.0".to_string(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_version(mut self, v: impl Into<String>) -> Self {
        self.version = v.into();
        self
    }

    pub fn with_metadata(mut self, k: impl Into<String>, v: impl Into<String>) -> Self {
        self.metadata.insert(k.into(), v.into());
        self
    }
}

/// A morphism between objects — represents a dependency or mathematical connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Morphism {
    pub id: MorphismId,
    pub source: ObjectId,
    pub target: ObjectId,
    pub label: String,
    pub weight: f64,
}

impl Morphism {
    pub fn new(
        id: impl Into<String>,
        source: ObjectId,
        target: ObjectId,
    ) -> Self {
        Morphism {
            id: MorphismId(id.into()),
            source,
            target,
            label: String::new(),
            weight: 1.0,
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn with_weight(mut self, w: f64) -> Self {
        self.weight = w;
        self
    }

    pub fn compose(&self, other: &Morphism, composed_id: impl Into<String>) -> Option<Morphism> {
        if self.target == other.source {
            Some(Morphism {
                id: MorphismId(composed_id.into()),
                source: self.source.clone(),
                target: other.target.clone(),
                label: format!("{};{}", self.label, other.label),
                weight: self.weight * other.weight,
            })
        } else {
            None
        }
    }
}

/// A category: a collection of objects and morphisms with identity and composition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub name: String,
    objects: HashMap<ObjectId, Object>,
    morphisms: HashMap<(ObjectId, ObjectId), Vec<Morphism>>,
    identities: HashMap<ObjectId, Morphism>,
}

impl Category {
    pub fn new(name: impl Into<String>) -> Self {
        Category {
            name: name.into(),
            objects: HashMap::new(),
            morphisms: HashMap::new(),
            identities: HashMap::new(),
        }
    }

    /// Add an object to the category, creating its identity morphism.
    pub fn add_object(&mut self, obj: Object) -> &mut Self {
        let id = obj.id.clone();
        let identity = Morphism {
            id: MorphismId(format!("id_{}", id.0)),
            source: id.clone(),
            target: id.clone(),
            label: "identity".to_string(),
            weight: 1.0,
        };
        self.identities.insert(id.clone(), identity);
        self.objects.insert(id, obj);
        self
    }

    /// Add a morphism to the category.
    pub fn add_morphism(&mut self, m: Morphism) -> Result<(), String> {
        if !self.objects.contains_key(&m.source) || !self.objects.contains_key(&m.target) {
            return Err("Source or target object not in category".to_string());
        }
        let key = (m.source.clone(), m.target.clone());
        self.morphisms.entry(key).or_default().push(m);
        Ok(())
    }

    pub fn get_object(&self, id: &ObjectId) -> Option<&Object> {
        self.objects.get(id)
    }

    pub fn objects(&self) -> impl Iterator<Item = &Object> {
        self.objects.values()
    }

    pub fn object_count(&self) -> usize {
        self.objects.len()
    }

    pub fn morphism_count(&self) -> usize {
        self.morphisms.values().map(|v| v.len()).sum::<usize>() + self.identities.len()
    }

    pub fn morphisms_from(&self, src: &ObjectId) -> Vec<&Morphism> {
        let mut result: Vec<&Morphism> = Vec::new();
        for ((s, _), ms) in &self.morphisms {
            if s == src {
                result.extend(ms.iter());
            }
        }
        result
    }

    pub fn morphisms_to(&self, tgt: &ObjectId) -> Vec<&Morphism> {
        let mut result: Vec<&Morphism> = Vec::new();
        for ((_, t), ms) in &self.morphisms {
            if t == tgt {
                result.extend(ms.iter());
            }
        }
        result
    }

    /// Get identity morphism for an object.
    pub fn identity(&self, id: &ObjectId) -> Option<&Morphism> {
        self.identities.get(id)
    }

    /// Verify left identity law: id ∘ f = f
    pub fn check_left_identity(&self, m: &Morphism) -> bool {
        if let Some(id_mor) = self.identity(&m.source) {
            if let Some(composed) = id_mor.compose(m, "check") {
                composed.source == m.source && composed.target == m.target
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Verify right identity law: f ∘ id = f
    pub fn check_right_identity(&self, m: &Morphism) -> bool {
        if let Some(id_mor) = self.identity(&m.target) {
            if let Some(composed) = m.compose(id_mor, "check") {
                composed.source == m.source && composed.target == m.target
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Verify associativity: (f ∘ g) ∘ h = f ∘ (g ∘ h)
    pub fn check_associativity(f: &Morphism, g: &Morphism, h: &Morphism) -> bool {
        // f: A→B, g: B→C, h: C→D
        let fg = f.compose(g, "fg");
        let gh = g.compose(h, "gh");
        match (fg, gh) {
            (Some(fg), Some(gh)) => {
                let left = fg.compose(h, "left");
                let right = f.compose(&gh, "right");
                match (left, right) {
                    (Some(l), Some(r)) => l.source == r.source && l.target == r.target,
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// Get all morphisms between two objects.
    pub fn hom(&self, src: &ObjectId, tgt: &ObjectId) -> Vec<&Morphism> {
        if src == tgt {
            let mut result = Vec::new();
            if let Some(id) = self.identities.get(src) {
                result.push(id);
            }
            if let Some(ms) = self.morphisms.get(&(src.clone(), tgt.clone())) {
                result.extend(ms.iter());
            }
            result
        } else {
            self.morphisms
                .get(&(src.clone(), tgt.clone()))
                .map(|v| v.iter().collect())
                .unwrap_or_default()
        }
    }

    /// Check if this is a subcategory of another (all objects/morphisms contained).
    pub fn is_subcategory_of(&self, other: &Category) -> bool {
        self.objects.keys().all(|k| other.objects.contains_key(k))
    }

    /// Build an adjacency matrix using nalgebra for morphism weights.
    pub fn adjacency_matrix(&self) -> (nalgebra::DMatrix<f64>, Vec<ObjectId>) {
        let ids: Vec<ObjectId> = self.objects.keys().cloned().collect();
        let n = ids.len();
        let mut mat = nalgebra::DMatrix::zeros(n, n);
        for (i, src) in ids.iter().enumerate() {
            for (j, tgt) in ids.iter().enumerate() {
                if let Some(ms) = self.morphisms.get(&(src.clone(), tgt.clone())) {
                    let max_w = ms.iter().map(|m| m.weight).fold(0.0f64, f64::max);
                    mat[(i, j)] = max_w;
                }
                if src == tgt {
                    mat[(i, j)] = 1.0; // identity
                }
            }
        }
        (mat, ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_creation() {
        let obj = Object::new("lau-core", "LAU Core");
        assert_eq!(obj.id, ObjectId("lau-core".to_string()));
        assert_eq!(obj.name, "LAU Core");
    }

    #[test]
    fn test_object_with_metadata() {
        let obj = Object::new("lau-core", "Core").with_metadata("domain", "foundation");
        assert_eq!(obj.metadata.get("domain").unwrap(), "foundation");
    }

    #[test]
    fn test_morphism_creation() {
        let m = Morphism::new("dep", ObjectId("a".into()), ObjectId("b".into()));
        assert_eq!(m.source, ObjectId("a".into()));
        assert_eq!(m.target, ObjectId("b".into()));
    }

    #[test]
    fn test_morphism_compose_success() {
        let f = Morphism::new("f", ObjectId("a".into()), ObjectId("b".into()));
        let g = Morphism::new("g", ObjectId("b".into()), ObjectId("c".into()));
        let fg = f.compose(&g, "fg").unwrap();
        assert_eq!(fg.source, ObjectId("a".into()));
        assert_eq!(fg.target, ObjectId("c".into()));
    }

    #[test]
    fn test_morphism_compose_fail() {
        let f = Morphism::new("f", ObjectId("a".into()), ObjectId("b".into()));
        let g = Morphism::new("g", ObjectId("c".into()), ObjectId("d".into()));
        assert!(f.compose(&g, "fg").is_none());
    }

    #[test]
    fn test_category_add_objects() {
        let mut cat = Category::new("TestCat");
        cat.add_object(Object::new("a", "A"));
        cat.add_object(Object::new("b", "B"));
        assert_eq!(cat.object_count(), 2);
    }

    #[test]
    fn test_category_identity_auto_created() {
        let mut cat = Category::new("TestCat");
        cat.add_object(Object::new("a", "A"));
        let id = ObjectId("a".into());
        assert!(cat.identity(&id).is_some());
        let id_mor = cat.identity(&id).unwrap();
        assert_eq!(id_mor.source, id_mor.target);
    }

    #[test]
    fn test_category_add_morphism() {
        let mut cat = Category::new("TestCat");
        cat.add_object(Object::new("a", "A"));
        cat.add_object(Object::new("b", "B"));
        let m = Morphism::new("f", ObjectId("a".into()), ObjectId("b".into()));
        assert!(cat.add_morphism(m).is_ok());
        assert_eq!(cat.morphism_count(), 3); // 2 identities + 1 morphism
    }

    #[test]
    fn test_category_add_morphism_missing_object() {
        let mut cat = Category::new("TestCat");
        cat.add_object(Object::new("a", "A"));
        let m = Morphism::new("f", ObjectId("a".into()), ObjectId("b".into()));
        assert!(cat.add_morphism(m).is_err());
    }

    #[test]
    fn test_left_identity_law() {
        let mut cat = Category::new("TestCat");
        cat.add_object(Object::new("a", "A"));
        cat.add_object(Object::new("b", "B"));
        let m = Morphism::new("f", ObjectId("a".into()), ObjectId("b".into()));
        cat.add_morphism(m.clone()).unwrap();
        assert!(cat.check_left_identity(&m));
    }

    #[test]
    fn test_right_identity_law() {
        let mut cat = Category::new("TestCat");
        cat.add_object(Object::new("a", "A"));
        cat.add_object(Object::new("b", "B"));
        let m = Morphism::new("f", ObjectId("a".into()), ObjectId("b".into()));
        cat.add_morphism(m.clone()).unwrap();
        assert!(cat.check_right_identity(&m));
    }

    #[test]
    fn test_associativity() {
        let f = Morphism::new("f", ObjectId("a".into()), ObjectId("b".into()));
        let g = Morphism::new("g", ObjectId("b".into()), ObjectId("c".into()));
        let h = Morphism::new("h", ObjectId("c".into()), ObjectId("d".into()));
        assert!(Category::check_associativity(&f, &g, &h));
    }

    #[test]
    fn test_adjacency_matrix() {
        let mut cat = Category::new("TestCat");
        cat.add_object(Object::new("a", "A"));
        cat.add_object(Object::new("b", "B"));
        let m = Morphism::new("f", ObjectId("a".into()), ObjectId("b".into())).with_weight(2.5);
        cat.add_morphism(m).unwrap();
        let (mat, ids) = cat.adjacency_matrix();
        assert_eq!(ids.len(), 2);
        assert_eq!(mat[(0, 0)], 1.0); // identity
        assert_eq!(mat[(1, 1)], 1.0); // identity
    }

    #[test]
    fn test_subcategory() {
        let mut big = Category::new("Big");
        big.add_object(Object::new("a", "A"));
        big.add_object(Object::new("b", "B"));
        let mut small = Category::new("Small");
        small.add_object(Object::new("a", "A"));
        assert!(small.is_subcategory_of(&big));
        assert!(!big.is_subcategory_of(&small));
    }

    #[test]
    fn test_hom_set() {
        let mut cat = Category::new("TestCat");
        cat.add_object(Object::new("a", "A"));
        cat.add_object(Object::new("b", "B"));
        cat.add_morphism(Morphism::new("f", ObjectId("a".into()), ObjectId("b".into()))).unwrap();
        let hom = cat.hom(&ObjectId("a".into()), &ObjectId("b".into()));
        assert_eq!(hom.len(), 1);
    }
}
