use crate::core::network::protocol::nbt::{byte, compound, compound_node, list, short, string, CompoundBuilder, NBTNode, NBT, TAG_COMPOUND_ID, TAG_STRING_ID};
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

    pub fn nbt_builder(&mut self) -> CompoundBuilder<'_> {
        let nbt = self.tag_compound.get_or_insert(NBT::default());
        nbt.builder()
    }

    pub fn name(mut self, name: &str) -> Self {
        let mut nbt = self.nbt_builder();
        nbt.get_or_insert_compound("display", [
            string("Name", name)
        ]);
        self
    }

    pub fn lore(mut self, lore: &str) -> Self {
        let lines: Vec<NBTNode> = lore
            .lines()
            .map(|line| NBTNode::String(line.to_string()))
            .collect();

        let mut nbt = self.nbt_builder();
        nbt.get_or_insert_compound("display", [
            list::<TAG_STRING_ID>("Lore", lines),
        ]);
        self
    }

    pub fn enchant(mut self, id: i16, level: i16) -> Self {
        let mut nbt = self.nbt_builder();
        nbt.get_or_insert_list::<TAG_COMPOUND_ID>("ench", [
            compound_node([
                short("id", id),
                short("lvl", level),
            ])
        ]);
        self
    }

    pub fn unbreakable(mut self) -> Self {
        let mut nbt = self.nbt_builder();
        nbt.insert([
            byte("Unbreakable", 1)
        ]);
        self
    }

    pub fn hide_all_flags(mut self) -> Self {
        let mut nbt = self.nbt_builder();
        nbt.insert([
            byte("HideFlags", 127)
        ]);
        self
    }

    pub fn skyblock_id(mut self, id: &str) -> Self {
        let mut nbt = self.nbt_builder();
        nbt.get_or_insert_compound("ExtraAttributes", [
            string("id", id)
        ]);
        self
    }

    pub fn skull_owner(
        mut self,
        uuid: uuid::Uuid,
        skin: PlayerSkin,
    ) -> Self {
        let mut nbt = self.nbt_builder();
        nbt.get_or_insert_compound("SkullOwner", [
            string("Id", uuid.hyphenated().to_string()),
            compound("Properties", [
                list::<TAG_COMPOUND_ID>("textures", [
                    compound_node([
                        string("Value", skin.texture.clone()),
                    ]),
                ])
            ])
        ]);
        self
    }
}
