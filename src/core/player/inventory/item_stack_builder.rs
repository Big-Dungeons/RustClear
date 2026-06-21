use crate::core::network::protocol::nbt::{NBTNode, NBT, TAG_COMPOUND_ID, TAG_STRING_ID};
use crate::core::player::inventory::item_stack::ItemStack;
use crate::dungeon::rng::DHashMap;
use bevy::utils::default;
use crate::core::player::PlayerSkin;

impl ItemStack {
    pub fn item_id(mut self, item_id: usize) -> Self {
        self.item = item_id as i16;
        self
    }

    pub fn stack(mut self, amount: usize) -> Self {
        self.item = amount as i16;
        self
    }

    pub fn metadata(mut self, metadata: usize) -> Self {
        self.metadata = metadata as i16;
        self
    }

    pub fn nbt_get_or_insert(&mut self) -> &mut NBT {
        self.tag_compound.get_or_insert(NBT::default())
    }

    pub fn name(mut self, name: &str) -> Self {
        let nbt = self.nbt_get_or_insert();
        with_compound(&mut nbt.nodes, "display", |nodes| {
            nodes.insert("Name".into(), NBTNode::String(name.into()));
        });
        self
    }

    pub fn lore(mut self, lore: &str) -> Self {
        let nbt = self.nbt_get_or_insert();
        with_compound(&mut nbt.nodes, "display", |nodes| {
            let list = lore
                .lines()
                .map(|line| NBTNode::String(line.to_string()))
                .collect();

            nodes.insert(
                "Lore".into(),
                NBTNode::List {
                    type_id: TAG_STRING_ID,
                    children: list,
                },
            );
        });
        self
    }

    pub fn enchant(mut self, id: i16, lvl: i16) -> Self {
        let nbt = self.nbt_get_or_insert();
        nbt.nodes
            .entry("ench".into())
            .or_insert(NBTNode::List { type_id: TAG_COMPOUND_ID, children: vec![] });

        if let Some(NBTNode::List { children, .. }) = nbt.nodes.get_mut("ench") {
            children.push(NBTNode::Compound({
                let mut map = DHashMap::default();
                map.insert("id".into(), NBTNode::Short(id));
                map.insert("lvl".into(), NBTNode::Short(lvl));
                map
            }))
        }
        self
    }

    pub fn unbreakable(mut self) -> Self {
        let nbt = self.nbt_get_or_insert();
        nbt.nodes.insert("Unbreakable".into(), NBTNode::Byte(1));
        self
    }

    pub fn hide_all_flags(mut self) -> Self {
        let nbt = self.nbt_get_or_insert();
        nbt.nodes.insert("HideFlags".into(), NBTNode::Byte(127));
        self
    }

    pub fn skyblock_id(mut self, id: &str) -> Self {
        let nbt = self.nbt_get_or_insert();
        with_compound(&mut nbt.nodes, "ExtraAttributes", |nodes| {
            nodes.insert("id".into(), NBTNode::String(id.into()));
        });
        self
    }

    pub fn skull_owner(
        mut self,
        uuid: uuid::Uuid,
        skin: PlayerSkin,
    ) -> Self {
        let nbt = self.nbt_get_or_insert();
        // this sucks
        with_compound(&mut nbt.nodes, "SkullOwner", |nodes| {
            nodes.insert("Id".into(), NBTNode::String(uuid.hyphenated().to_string()));
            let mut map = DHashMap::default();
            map.insert("textures".into(), NBTNode::List {
                type_id: TAG_COMPOUND_ID,
                children: vec![
                    NBTNode::Compound({
                        let mut map = DHashMap::default();
                        map.insert("Value".into(), NBTNode::String(skin.texture));
                        map.insert("Signature".into(), NBTNode::String(skin._signature.unwrap_or_default()));
                        map
                    })
                ]
            });
            nodes.insert("Properties".into(), NBTNode::Compound(map));
        });
        self
    }
}

fn with_compound<F>(nodes: &mut DHashMap<String, NBTNode>, node: &str, func: F)
where
    F: FnOnce(&mut DHashMap<String, NBTNode>),
{
    nodes
        .entry(node.into())
        .or_insert(NBTNode::Compound(default()));
    // should probably assert or debug_assert
    if let Some(NBTNode::Compound(compound)) = nodes.get_mut(node) {
        func(compound);
    }
}
