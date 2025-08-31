use crate::net::packets::packet_serialize::PacketSerializable;
use crate::net::var_int::{var_int_size, VarInt};
use bytes::BytesMut;
use std::collections::HashMap;
use std::hash::Hash;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct AttributeMap {
    map: HashMap<Attribute, AttributeInstance>
}

#[derive(Clone, Debug)]
pub struct AttributeInstance {
    pub value: f64,
    pub modifiers: Vec<AttributeModifier>,
}
#[derive(Clone, Debug)]
pub struct AttributeModifier {
    pub id: Uuid,
    pub amount: f64,
    pub operation: i8,
}

impl AttributeMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    
    pub fn insert(&mut self, attribute: Attribute, value: f64) {
        self.map.insert(attribute, AttributeInstance {
            value,
            modifiers: Vec::new(),
        });
    }

    pub fn add_modify(&mut self, attribute: Attribute, modifier: AttributeModifier) {
        if let Some(instance) = self.map.get_mut(&attribute) {
            instance.modifiers.push(modifier);
        }
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum Attribute {
    MaxHealth,
    KnockbackResistance,
    MovementSpeed,

    // these might be useless
    AttackDamage,
    FollowRange,
    HorseJumpStrength,
    SpawnReinforcements
}

impl Attribute {
    const fn id(&self) -> &str {
        match self {
            Attribute::MaxHealth => "generic.maxHealth",
            Attribute::KnockbackResistance => "generic.knockbackResistance",
            Attribute::MovementSpeed => "generic.movementSpeed",
            Attribute::AttackDamage => "generic.attackDamage",
            Attribute::FollowRange => "generic.followRange",
            Attribute::HorseJumpStrength => "horse.jumpStrength",
            Attribute::SpawnReinforcements => "zombie.spawnReinforcements",
        }
    }
}

impl PacketSerializable for AttributeMap {
    fn write_size(&self) -> usize {
        let mut size = (self.map.len() as i32).write_size();
        for (attribute_type, value) in &self.map {
            size +=
                attribute_type.id().write_size() +
                value.value.write_size() +
                var_int_size(value.modifiers.len() as i32);
            
            for modifier in &value.modifiers {
                size += 
                    modifier.id.write_size() +
                    modifier.amount.write_size() +
                    modifier.operation.write_size()
            }
        }
        size
    }
    fn write(&self, buf: &mut BytesMut) {
        (self.map.len() as i32).write(buf);

        for (attribute_type, value) in &self.map {
            attribute_type.id().write(buf);
            value.value.write(buf);
            VarInt(value.modifiers.len() as i32).write(buf);
            for modifier in &value.modifiers {
                modifier.id.write(buf);
                modifier.amount.write(buf);
                modifier.operation.write(buf);
            }
        }
    }
}