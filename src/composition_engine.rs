//! Composition engine — automatically compose crate functionality via category theory.

use serde::{Deserialize, Serialize};
use crate::category::{Category, Morphism, ObjectId, Object};
use crate::functor::Functor;
use crate::limit_colimit;
use std::collections::HashMap;

/// A composition plan describing how to combine crate functionality.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionPlan {
    pub name: String,
    pub steps: Vec<CompositionStep>,
    pub input_types: Vec<String>,
    pub output_types: Vec<String>,
}

/// A single step in a composition plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionStep {
    pub crate_name: String,
    pub operation: String,
    pub inputs: Vec<String>,
    pub output: String,
}

/// The composition engine that uses category theory to plan and validate compositions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionEngine {
    pub category: Category,
    pub functors: HashMap<String, Functor>,
    pub(crate) plans: HashMap<String, CompositionPlan>,
}

impl CompositionEngine {
    pub fn new(category: Category) -> Self {
        CompositionEngine {
            category,
            functors: HashMap::new(),
            plans: HashMap::new(),
        }
    }

    /// Register a functor for translating between sub-categories.
    pub fn register_functor(&mut self, functor: Functor) -> &mut Self {
        self.functors.insert(functor.name.clone(), functor);
        self
    }

    /// Find a path from source crate to target crate through morphisms.
    pub fn find_path(&self, source: &ObjectId, target: &ObjectId) -> Option<Vec<ObjectId>> {
        // BFS on the dependency graph
        let mut visited: HashMap<ObjectId, bool> = HashMap::new();
        let mut queue: Vec<(ObjectId, Vec<ObjectId>)> = vec![(source.clone(), vec![source.clone()])];
        visited.insert(source.clone(), true);

        while let Some((current, path)) = queue.pop() {
            if &current == target {
                return Some(path);
            }
            for m in self.category.morphisms_from(&current) {
                if !visited.contains_key(&m.target) {
                    visited.insert(m.target.clone(), true);
                    let mut new_path = path.clone();
                    new_path.push(m.target.clone());
                    queue.push((m.target.clone(), new_path));
                }
            }
        }
        None
    }

    /// Find all crates that depend on a given crate (transitive closure).
    pub fn dependents(&self, crate_id: &ObjectId) -> Vec<ObjectId> {
        let mut result = Vec::new();
        let mut visited: HashMap<ObjectId, bool> = HashMap::new();
        let mut queue = vec![crate_id.clone()];

        while let Some(current) = queue.pop() {
            for m in self.category.morphisms_to(&current) {
                if !visited.contains_key(&m.source) {
                    visited.insert(m.source.clone(), true);
                    result.push(m.source.clone());
                    queue.push(m.source.clone());
                }
            }
        }
        result
    }

    /// Find all dependencies of a crate (transitive).
    pub fn dependencies(&self, crate_id: &ObjectId) -> Vec<ObjectId> {
        let mut result = Vec::new();
        let mut visited: HashMap<ObjectId, bool> = HashMap::new();
        let mut queue = vec![crate_id.clone()];

        while let Some(current) = queue.pop() {
            for m in self.category.morphisms_from(&current) {
                if !visited.contains_key(&m.target) {
                    visited.insert(m.target.clone(), true);
                    result.push(m.target.clone());
                    queue.push(m.target.clone());
                }
            }
        }
        result
    }

    /// Compose two crates via their product (parallel composition).
    pub fn compose_parallel(
        &mut self,
        a: &ObjectId,
        b: &ObjectId,
    ) -> Result<ObjectId, String> {
        let result = limit_colimit::product(&mut self.category, a, b)?;
        Ok(result.limit_object.id)
    }

    /// Compose two crates via coproduct (alternative composition).
    pub fn compose_alternative(
        &mut self,
        a: &ObjectId,
        b: &ObjectId,
    ) -> Result<ObjectId, String> {
        let result = limit_colimit::coproduct(&mut self.category, a, b)?;
        Ok(result.colimit_object.id)
    }

    /// Create a composition plan by chaining crates.
    pub fn create_plan(
        &mut self,
        name: impl Into<String>,
        steps: Vec<CompositionStep>,
    ) -> Result<&CompositionPlan, String> {
        let plan = CompositionPlan {
            name: name.into(),
            input_types: steps.first().map(|s| s.inputs.clone()).unwrap_or_default(),
            output_types: steps
                .last()
                .map(|s| vec![s.output.clone()])
                .unwrap_or_default(),
            steps,
        };
        let plan_name = plan.name.clone();
        self.plans.insert(plan_name.clone(), plan);
        Ok(self.plans.get(&plan_name).unwrap())
    }

    /// Get a composition plan by name.
    pub fn get_plan(&self, name: &str) -> Option<&CompositionPlan> {
        self.plans.get(name)
    }

    /// Validate a composition plan: check that all referenced crates exist.
    pub fn validate_plan(&self, plan: &CompositionPlan) -> Result<bool, String> {
        for step in &plan.steps {
            if self.category.get_object(&ObjectId(step.crate_name.clone())).is_none() {
                return Err(format!("Crate '{}' not found in category", step.crate_name));
            }
        }
        Ok(true)
    }

    /// Compute the transitive closure of the dependency graph as an adjacency matrix.
    pub fn transitive_closure(&self) -> (nalgebra::DMatrix<f64>, Vec<ObjectId>) {
        let (mut mat, ids) = self.category.adjacency_matrix();
        let n = ids.len();
        // Floyd-Warshall for transitive closure
        for k in 0..n {
            for i in 0..n {
                for j in 0..n {
                    if mat[(i, k)] > 0.0 && mat[(k, j)] > 0.0 {
                        mat[(i, j)] = mat[(i, j)].max(mat[(i, k)] * mat[(k, j)]);
                    }
                }
            }
        }
        (mat, ids)
    }

    /// Find shared dependencies between two crates.
    pub fn shared_dependencies(&self, a: &ObjectId, b: &ObjectId) -> Vec<ObjectId> {
        let deps_a: std::collections::HashSet<_> = self.dependencies(a).into_iter().collect();
        let deps_b: std::collections::HashSet<_> = self.dependencies(b).into_iter().collect();
        deps_a.intersection(&deps_b).cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_engine() -> CompositionEngine {
        let mut cat = Category::new("SuperInstance");
        cat.add_object(Object::new("lau-core", "Core"));
        cat.add_object(Object::new("lau-agents", "Agents"));
        cat.add_object(Object::new("lau-security", "Security"));
        cat.add_object(Object::new("lau-network", "Network"));
        cat.add_object(Object::new("lau-math", "Math"));

        cat.add_morphism(Morphism::new("d1", ObjectId("lau-agents".into()), ObjectId("lau-core".into()))).unwrap();
        cat.add_morphism(Morphism::new("d2", ObjectId("lau-security".into()), ObjectId("lau-core".into()))).unwrap();
        cat.add_morphism(Morphism::new("d3", ObjectId("lau-network".into()), ObjectId("lau-security".into()))).unwrap();
        cat.add_morphism(Morphism::new("d4", ObjectId("lau-agents".into()), ObjectId("lau-math".into()))).unwrap();

        CompositionEngine::new(cat)
    }

    #[test]
    fn test_engine_creation() {
        let engine = setup_engine();
        assert_eq!(engine.category.object_count(), 5);
    }

    #[test]
    fn test_find_path_direct() {
        let engine = setup_engine();
        let path = engine.find_path(&ObjectId("lau-agents".into()), &ObjectId("lau-core".into()));
        assert!(path.is_some());
        assert!(path.unwrap().len() >= 2);
    }

    #[test]
    fn test_find_path_indirect() {
        let engine = setup_engine();
        let path = engine.find_path(&ObjectId("lau-network".into()), &ObjectId("lau-core".into()));
        assert!(path.is_some());
    }

    #[test]
    fn test_find_path_none() {
        let engine = setup_engine();
        let path = engine.find_path(&ObjectId("lau-core".into()), &ObjectId("lau-network".into()));
        assert!(path.is_none());
    }

    #[test]
    fn test_dependents() {
        let engine = setup_engine();
        let deps = engine.dependents(&ObjectId("lau-core".into()));
        assert!(deps.contains(&ObjectId("lau-agents".into())));
        assert!(deps.contains(&ObjectId("lau-security".into())));
    }

    #[test]
    fn test_dependencies() {
        let engine = setup_engine();
        let deps = engine.dependencies(&ObjectId("lau-network".into()));
        assert!(deps.contains(&ObjectId("lau-security".into())));
        assert!(deps.contains(&ObjectId("lau-core".into())));
    }

    #[test]
    fn test_compose_parallel() {
        let mut engine = setup_engine();
        let result = engine.compose_parallel(&ObjectId("lau-agents".into()), &ObjectId("lau-security".into()));
        assert!(result.is_ok());
        assert!(result.unwrap().0.contains("×"));
    }

    #[test]
    fn test_compose_alternative() {
        let mut engine = setup_engine();
        let result = engine.compose_alternative(&ObjectId("lau-agents".into()), &ObjectId("lau-security".into()));
        assert!(result.is_ok());
        assert!(result.unwrap().0.contains("+"));
    }

    #[test]
    fn test_create_plan() {
        let mut engine = setup_engine();
        let plan = engine.create_plan("test_plan", vec![
            CompositionStep {
                crate_name: "lau-agents".into(),
                operation: "process".into(),
                inputs: vec!["data".into()],
                output: "processed".into(),
            },
        ]);
        assert!(plan.is_ok());
    }

    #[test]
    fn test_validate_plan() {
        let mut engine = setup_engine();
        engine.create_plan("test_plan", vec![
            CompositionStep {
                crate_name: "lau-agents".into(),
                operation: "process".into(),
                inputs: vec!["data".into()],
                output: "processed".into(),
            },
        ]).unwrap();
        let plan = engine.get_plan("test_plan").unwrap();
        assert!(engine.validate_plan(plan).is_ok());
    }

    #[test]
    fn test_validate_plan_missing_crate() {
        let engine = setup_engine();
        let plan = CompositionPlan {
            name: "bad".into(),
            steps: vec![CompositionStep {
                crate_name: "nonexistent".into(),
                operation: "op".into(),
                inputs: vec![],
                output: "out".into(),
            }],
            input_types: vec![],
            output_types: vec![],
        };
        assert!(engine.validate_plan(&plan).is_err());
    }

    #[test]
    fn test_transitive_closure() {
        let engine = setup_engine();
        let (mat, ids) = engine.transitive_closure();
        assert_eq!(ids.len(), 5);
    }

    #[test]
    fn test_shared_dependencies() {
        let engine = setup_engine();
        let shared = engine.shared_dependencies(&ObjectId("lau-agents".into()), &ObjectId("lau-network".into()));
        assert!(shared.contains(&ObjectId("lau-core".into())));
    }
}
