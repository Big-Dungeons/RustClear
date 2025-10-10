use crate::inventory::item_stack::ItemStack;

pub trait Item {
    
    fn get_item_stack(&self) -> ItemStack;
    
    // this is small workaround to not require needing a custom inventory implementation.
    fn can_move_in_inventory(&self) -> bool;
}

pub fn get_item_stack<T : Item>(item: &Option<T>) -> Option<ItemStack> {
    item.as_ref().map(|item| item.get_item_stack())
}