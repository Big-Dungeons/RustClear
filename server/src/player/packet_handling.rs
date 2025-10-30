use crate::inventory::menu::OpenContainer;
use crate::network::packets::packet::ProcessPacket;
use crate::network::protocol::play::clientbound::Chat;
use crate::network::protocol::play::serverbound;
use crate::network::protocol::play::serverbound::{ArmSwing, ChatMessage, ClickWindow, ClientSettings, ClientStatus, CreativeInventoryAction, HeldItemChange, PlayerAction, PlayerActionType, PlayerBlockPlacement, PlayerDigging, PlayerLook, PlayerPosition, PlayerPositionLook, PlayerUpdate, TabComplete, UseEntity};
use crate::player::player::{Player, PlayerExtension};
use crate::types::chat_component::ChatComponent;
use crate::types::direction::Direction;
use enumset::EnumSet;
use glam::IVec3;

impl ProcessPacket for serverbound::KeepAlive {
    fn process<P : PlayerExtension>(&self, _: &mut Player<P>) {
        // if player.last_keep_alive == self.id {
        //     if let Ok(since) = SystemTime::now().duration_since(UNIX_EPOCH) {
        //         let since = since.as_millis() as i32 - player.last_keep_alive;
        //         player.ping = (player.ping * 3 + since) / 4;
        //         println!("Ping: {}", player.ping);
        //     }
        // }
    }
}

impl ProcessPacket for ChatMessage {
    fn process<P : PlayerExtension>(&self, player: &mut Player<P>) {
        player.write_packet(&Chat {
            component: ChatComponent::new(String::from_utf8(self.message.as_bytes().to_vec()).unwrap()),
            chat_type: 0,
        })
    }
}

impl ProcessPacket for UseEntity {
    fn process<P : PlayerExtension>(&self, player: &mut Player<P>) {
        if let Some(index) = player.world_mut().entity_map.get(&self.entity_id.0) {
            let entity = &mut player.world_mut().entities[*index];
            entity.interact(player, self.action)
        }
    }
}

// I don't know if any implementation will be needed,
// but just in case imma keep it here
impl ProcessPacket for PlayerUpdate {
    fn process<P: PlayerExtension>(&self, _: &mut Player<P>) {

    }
}

// anti cheat stuff vvv important to do for all 3

impl ProcessPacket for PlayerPosition {
    fn process<P : PlayerExtension>(&self, player: &mut Player<P>) {
        player.position.x = self.x;
        player.position.y = self.y;
        player.position.z = self.z;
    }
}

impl ProcessPacket for PlayerLook {
    fn process<P : PlayerExtension>(&self, player: &mut Player<P>) {
        player.yaw = self.yaw;
        player.pitch = self.pitch;
    }
}

impl ProcessPacket for PlayerPositionLook {
    fn process<P : PlayerExtension>(&self, player: &mut Player<P>) {
        player.position.x = self.x;
        player.position.y = self.y;
        player.position.z = self.z;
        player.yaw = self.yaw;
        player.pitch = self.pitch;
    }
}

impl ProcessPacket for PlayerDigging {
    fn process<P : PlayerExtension>(&self, player: &mut Player<P>) {
        P::dig(player, self.position.0, &self.action);
    }
}

pub struct BlockInteractResult {
    pub position: IVec3,
    pub direction: Direction,
}

impl ProcessPacket for PlayerBlockPlacement {
    fn process<P : PlayerExtension>(&self, player: &mut Player<P>) {
        if !player.sent_block_placement {
            player.sent_block_placement = true;
            
            let block_hit_result = if self.position.y.is_negative() {
                None
            } else {
                Some(BlockInteractResult {
                    position: *self.position,
                    direction: match self.placed_direction {
                        0 => Direction::Down,
                        1 => Direction::Up,
                        2 => Direction::North,
                        3 => Direction::South,
                        4 => Direction::West,
                        5 => Direction::East,
                        _ => unreachable!()
                    },
                })
            };
            P::interact(player, self.item_stack.clone(), block_hit_result);
        }
    }
}

impl ProcessPacket for HeldItemChange {
    fn process<P : PlayerExtension>(&self, player: &mut Player<P>) {
        // warn player if invalid packets
        let item_slot = self.slot_id.clamp(0, 8) as u8;
        player.held_slot = item_slot;
    }
}

// will be useful if we want to add stuff like mage beam
impl ProcessPacket for ArmSwing {
    fn process<P: PlayerExtension>(&self, _: &mut Player<P>) {
        // lc
    }
}

impl ProcessPacket for PlayerAction {
    fn process<P : PlayerExtension>(&self, player: &mut Player<P>) {
        match self.action {
            PlayerActionType::StartSneaking => player.is_sneaking = true,
            PlayerActionType::StopSneaking => player.is_sneaking = false,
            _ => {}
        }
    }
}

impl ProcessPacket for serverbound::CloseWindow {
    fn process<P : PlayerExtension>(&self, player: &mut Player<P>) {
        player.open_container(OpenContainer::None)
        // player.open_ui(UI::None)
    }
}

impl ProcessPacket for ClickWindow {
    fn process<P : PlayerExtension>(&self, player: &mut Player<P>) {
        let container = unsafe { player.open_container.get().as_mut().unwrap() };
        container.click_window(player, self);
    }
}

impl ProcessPacket for serverbound::ConfirmTransaction {
    fn process<P: PlayerExtension>(&self, _: &mut Player<P>) {
        // anti cheat
    }
}

impl ProcessPacket for CreativeInventoryAction {
    fn process<P: PlayerExtension>(&self, player: &mut Player<P>) {
        // println!("{self:?}")
    }
}

impl ProcessPacket for TabComplete {
    fn process<P: PlayerExtension>(&self, _: &mut Player<P>) {
        // for commands, we should have some tree system instead of recreating the mess that is mc 1.8.9 CommandBase
    }
    // fn process<P : PlayerExtension>(&self, player: &mut Player<P>) {
    //     if !self.message.starts_with("/") {
    //         return;
    //     }
    //     let parts: Vec<&str> = self.message.split_whitespace().collect();
    //     let command_name = parts[0].strip_prefix("/").unwrap();
    // 
    //     if command_name.is_empty() {
    //         player.write_packet(&TabCompleteReply {
    //             matches: Command::list().iter().map(|cmd| format!("/{}", cmd.name())).collect(),
    //         });
    //         return
    //     }
    // 
    //     if let Some(command) = Command::find(command_name) {
    //         let args = &parts[1..];
    // 
    //         let next_arg = self.message.ends_with(' ');
    // 
    //         if args.is_empty() && !next_arg {
    //             // user input a valid command but has not hit space, so we shouldn't provide any completions.
    //             // there might be a better way to do this somewhere else but idk atm.
    //             return;
    //         }
    // 
    //         let current_arg = if next_arg {
    //             args.len()
    //         } else {
    //             args.len().saturating_sub(1)
    //         };
    // 
    //         let command_args = command.args(player.world_mut(), player);
    // 
    //         if current_arg >= command_args.len() {
    //             // user has input too many arguments; so we just return here.
    //             return;
    //         }
    // 
    //         let completions = {
    //             let arg = &command_args.get(current_arg);
    //             if arg.is_none() { return; }
    //             &arg.unwrap().completions
    //         };
    // 
    //         let matches: Vec<String> = if next_arg || args.is_empty() {
    //             completions.to_vec()
    //         } else {
    //             completions.iter().filter(|cmp| cmp.starts_with(args.last().unwrap_or(&""))).cloned().collect()
    //         };
    // 
    //         player.write_packet(&TabCompleteReply {
    //             matches
    //         });
    //     } else {
    //         let commands = Command::list().iter().filter(|cmd| cmd.name().starts_with(command_name)).map(|cmd| format!("/{}", cmd.name())).collect();
    //         player.write_packet(&TabCompleteReply {
    //             matches: commands
    //         });
    //     }
    // }
}

impl ProcessPacket for ClientSettings {
    fn process<P: PlayerExtension>(&self, player: &mut Player<P>) {
        if player.metadata.layers.as_u8() != self.skin_parts {
            player.metadata.layers = EnumSet::from_u8(self.skin_parts);
            player.dirty_metadata = true;
        }
    }
}

impl ProcessPacket for ClientStatus {
    fn process<P : PlayerExtension>(&self, player: &mut Player<P>) {
        if let ClientStatus::OpenInventory = self {
            player.open_container(OpenContainer::Inventory)
        }
    }
}