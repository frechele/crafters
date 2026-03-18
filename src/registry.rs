use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct MaterialId(pub u16);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ItemId(pub u16);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct EntityTypeId(pub u16);

impl MaterialId {
    pub const EMPTY: MaterialId = MaterialId(0);
}

#[derive(Clone, Debug)]
pub struct MaterialDef {
    pub name: String,
    pub id: MaterialId,
    pub walkable: bool,
    pub player_walkable: bool,
    pub arrow_walkable: bool,
}

#[derive(Clone, Debug)]
pub struct ItemDef {
    pub name: String,
    pub id: ItemId,
    pub max: i32,
    pub initial: i32,
}

#[derive(Clone, Debug)]
pub struct Registry {
    pub materials: Vec<MaterialDef>,
    pub items: Vec<ItemDef>,
    material_by_name: HashMap<String, MaterialId>,
    item_by_name: HashMap<String, ItemId>,
}

impl Registry {
    pub fn new(materials: Vec<MaterialDef>, items: Vec<ItemDef>) -> Self {
        let material_by_name = materials
            .iter()
            .map(|m| (m.name.clone(), m.id))
            .collect();
        let item_by_name = items.iter().map(|i| (i.name.clone(), i.id)).collect();
        Self {
            materials,
            items,
            material_by_name,
            item_by_name,
        }
    }

    pub fn material_id(&self, name: &str) -> Option<MaterialId> {
        self.material_by_name.get(name).copied()
    }

    pub fn item_id(&self, name: &str) -> Option<ItemId> {
        self.item_by_name.get(name).copied()
    }

    pub fn material_name(&self, id: MaterialId) -> &str {
        &self.materials[id.0 as usize - 1].name
    }

    pub fn item_name(&self, id: ItemId) -> &str {
        &self.items[id.0 as usize].name
    }

    pub fn material_count(&self) -> usize {
        self.materials.len()
    }

    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    pub fn walkable_materials(&self) -> Vec<MaterialId> {
        self.materials
            .iter()
            .filter(|m| m.walkable)
            .map(|m| m.id)
            .collect()
    }

    pub fn player_walkable_materials(&self) -> Vec<MaterialId> {
        self.materials
            .iter()
            .filter(|m| m.player_walkable)
            .map(|m| m.id)
            .collect()
    }

    pub fn arrow_walkable_materials(&self) -> Vec<MaterialId> {
        self.materials
            .iter()
            .filter(|m| m.arrow_walkable)
            .map(|m| m.id)
            .collect()
    }
}
