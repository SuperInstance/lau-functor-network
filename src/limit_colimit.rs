//! Limits and colimits — products, coproducts, pullbacks, pushouts of crate compositions.

use serde::{Deserialize, Serialize};
use crate::category::{Category, Morphism, ObjectId, Object};

/// A cone over a diagram: an object (apex) with morphisms to each diagram object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cone {
    pub apex: ObjectId,
    pub projections: Vec<(ObjectId, Morphism)>, // (target, morphism from apex to target)
}

/// A cocone under a diagram: an object (nadir) with morphisms from each diagram object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cocone {
    pub nadir: ObjectId,
    pub injections: Vec<(ObjectId, Morphism)>, // (source, morphism from source to nadir)
}

/// Result of computing a limit or colimit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitResult {
    pub limit_object: Object,
    pub cone: Cone,
    pub is_universal: bool,
}

/// Result of computing a colimit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColimitResult {
    pub colimit_object: Object,
    pub cocone: Cocone,
    pub is_universal: bool,
}

/// Compute the product of two objects (binary product / limit of discrete diagram).
pub fn product(cat: &mut Category, a: &ObjectId, b: &ObjectId) -> Result<LimitResult, String> {
    let obj_a = cat.get_object(a).ok_or("Object A not found")?;
    let obj_b = cat.get_object(b).ok_or("Object B not found")?;

    let prod_id = ObjectId(format!("{}×{}", a.0, b.0));
    let prod_obj = Object::new(
        &prod_id.0,
        format!("Product({},{})", obj_a.name, obj_b.name),
    );
    cat.add_object(prod_obj.clone());

    let proj_a = Morphism::new(
        format!("π₁_{}×{}", a.0, b.0),
        prod_id.clone(),
        a.clone(),
    )
    .with_label("proj1");
    let proj_b = Morphism::new(
        format!("π₂_{}×{}", a.0, b.0),
        prod_id.clone(),
        b.clone(),
    )
    .with_label("proj2");

    cat.add_morphism(proj_a.clone())?;
    cat.add_morphism(proj_b.clone())?;

    let cone = Cone {
        apex: prod_id.clone(),
        projections: vec![(a.clone(), proj_a), (b.clone(), proj_b)],
    };

    Ok(LimitResult {
        limit_object: prod_obj,
        cone,
        is_universal: true,
    })
}

/// Compute the coproduct (sum) of two objects.
pub fn coproduct(cat: &mut Category, a: &ObjectId, b: &ObjectId) -> Result<ColimitResult, String> {
    let obj_a = cat.get_object(a).ok_or("Object A not found")?;
    let obj_b = cat.get_object(b).ok_or("Object B not found")?;

    let coprod_id = ObjectId(format!("{}+{}", a.0, b.0));
    let coprod_obj = Object::new(
        &coprod_id.0,
        format!("Coproduct({},{})", obj_a.name, obj_b.name),
    );
    cat.add_object(coprod_obj.clone());

    let inj_a = Morphism::new(
        format!("ι₁_{}+{}", a.0, b.0),
        a.clone(),
        coprod_id.clone(),
    )
    .with_label("inj1");
    let inj_b = Morphism::new(
        format!("ι₂_{}+{}", a.0, b.0),
        b.clone(),
        coprod_id.clone(),
    )
    .with_label("inj2");

    cat.add_morphism(inj_a.clone())?;
    cat.add_morphism(inj_b.clone())?;

    let cocone = Cocone {
        nadir: coprod_id.clone(),
        injections: vec![(a.clone(), inj_a), (b.clone(), inj_b)],
    };

    Ok(ColimitResult {
        colimit_object: coprod_obj,
        cocone,
        is_universal: true,
    })
}

/// Compute the equalizer of two parallel morphisms.
pub fn equalizer(
    cat: &mut Category,
    f: &Morphism,
    g: &Morphism,
) -> Result<LimitResult, String> {
    if f.source != g.source || f.target != g.target {
        return Err("Morphisms must be parallel (same source and target)".to_string());
    }
    let eq_id = ObjectId(format!("Eq({},{})", f.id.0, g.id.0));
    let eq_obj = Object::new(&eq_id.0, format!("Equalizer({},{})", f.id.0, g.id.0));
    cat.add_object(eq_obj.clone());

    let morph = Morphism::new(
        format!("eq_mor_{}", eq_id.0),
        eq_id.clone(),
        f.source.clone(),
    )
    .with_label("equalizer_morphism");
    cat.add_morphism(morph.clone())?;

    Ok(LimitResult {
        limit_object: eq_obj,
        cone: Cone {
            apex: eq_id,
            projections: vec![(f.source.clone(), morph)],
        },
        is_universal: true,
    })
}

/// Compute the pullback of two morphisms with a common target.
pub fn pullback(
    cat: &mut Category,
    f: &Morphism,
    g: &Morphism,
) -> Result<LimitResult, String> {
    if f.target != g.target {
        return Err("Morphisms must share a common target".to_string());
    }
    let pb_id = ObjectId(format!("PB({},{})", f.id.0, g.id.0));
    let pb_obj = Object::new(&pb_id.0, format!("Pullback({},{})", f.id.0, g.id.0));
    cat.add_object(pb_obj.clone());

    let proj_a = Morphism::new(format!("pb_π1_{}", pb_id.0), pb_id.clone(), f.source.clone());
    let proj_b = Morphism::new(format!("pb_π2_{}", pb_id.0), pb_id.clone(), g.source.clone());
    cat.add_morphism(proj_a.clone())?;
    cat.add_morphism(proj_b.clone())?;

    Ok(LimitResult {
        limit_object: pb_obj,
        cone: Cone {
            apex: pb_id,
            projections: vec![(f.source.clone(), proj_a), (g.source.clone(), proj_b)],
        },
        is_universal: true,
    })
}

/// Compute the pushout of two morphisms with a common source.
pub fn pushout(
    cat: &mut Category,
    f: &Morphism,
    g: &Morphism,
) -> Result<ColimitResult, String> {
    if f.source != g.source {
        return Err("Morphisms must share a common source".to_string());
    }
    let po_id = ObjectId(format!("PO({},{})", f.id.0, g.id.0));
    let po_obj = Object::new(&po_id.0, format!("Pushout({},{})", f.id.0, g.id.0));
    cat.add_object(po_obj.clone());

    let inj_a = Morphism::new(format!("po_ι1_{}", po_id.0), f.target.clone(), po_id.clone());
    let inj_b = Morphism::new(format!("po_ι2_{}", po_id.0), g.target.clone(), po_id.clone());
    cat.add_morphism(inj_a.clone())?;
    cat.add_morphism(inj_b.clone())?;

    Ok(ColimitResult {
        colimit_object: po_obj,
        cocone: Cocone {
            nadir: po_id,
            injections: vec![(f.target.clone(), inj_a), (g.target.clone(), inj_b)],
        },
        is_universal: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_category() -> Category {
        let mut cat = Category::new("TestCat");
        cat.add_object(Object::new("a", "A"));
        cat.add_object(Object::new("b", "B"));
        cat.add_object(Object::new("c", "C"));
        cat
    }

    #[test]
    fn test_product() {
        let mut cat = setup_category();
        let result = product(&mut cat, &ObjectId("a".into()), &ObjectId("b".into())).unwrap();
        assert_eq!(result.limit_object.id, ObjectId("a×b".into()));
        assert_eq!(result.cone.projections.len(), 2);
    }

    #[test]
    fn test_product_adds_to_category() {
        let mut cat = setup_category();
        product(&mut cat, &ObjectId("a".into()), &ObjectId("b".into())).unwrap();
        assert_eq!(cat.object_count(), 4); // a, b, c + a×b
    }

    #[test]
    fn test_coproduct() {
        let mut cat = setup_category();
        let result = coproduct(&mut cat, &ObjectId("a".into()), &ObjectId("b".into())).unwrap();
        assert_eq!(result.colimit_object.id, ObjectId("a+b".into()));
        assert_eq!(result.cocone.injections.len(), 2);
    }

    #[test]
    fn test_equalizer() {
        let mut cat = setup_category();
        let f = Morphism::new("f", ObjectId("a".into()), ObjectId("b".into()));
        let g = Morphism::new("g", ObjectId("a".into()), ObjectId("b".into()));
        let result = equalizer(&mut cat, &f, &g).unwrap();
        assert_eq!(result.limit_object.id.0, "Eq(f,g)");
    }

    #[test]
    fn test_equalizer_fails_non_parallel() {
        let mut cat = setup_category();
        let f = Morphism::new("f", ObjectId("a".into()), ObjectId("b".into()));
        let g = Morphism::new("g", ObjectId("a".into()), ObjectId("c".into()));
        assert!(equalizer(&mut cat, &f, &g).is_err());
    }

    #[test]
    fn test_pullback() {
        let mut cat = setup_category();
        let f = Morphism::new("f", ObjectId("a".into()), ObjectId("c".into()));
        let g = Morphism::new("g", ObjectId("b".into()), ObjectId("c".into()));
        let result = pullback(&mut cat, &f, &g).unwrap();
        assert_eq!(result.cone.projections.len(), 2);
    }

    #[test]
    fn test_pullback_fails_different_targets() {
        let mut cat = setup_category();
        let f = Morphism::new("f", ObjectId("a".into()), ObjectId("b".into()));
        let g = Morphism::new("g", ObjectId("b".into()), ObjectId("c".into()));
        assert!(pullback(&mut cat, &f, &g).is_err());
    }

    #[test]
    fn test_pushout() {
        let mut cat = setup_category();
        let f = Morphism::new("f", ObjectId("a".into()), ObjectId("b".into()));
        let g = Morphism::new("g", ObjectId("a".into()), ObjectId("c".into()));
        let result = pushout(&mut cat, &f, &g).unwrap();
        assert_eq!(result.cocone.injections.len(), 2);
    }

    #[test]
    fn test_pushout_fails_different_sources() {
        let mut cat = setup_category();
        let f = Morphism::new("f", ObjectId("a".into()), ObjectId("b".into()));
        let g = Morphism::new("g", ObjectId("b".into()), ObjectId("c".into()));
        assert!(pushout(&mut cat, &f, &g).is_err());
    }
}
