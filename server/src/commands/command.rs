use crate::commands::string_reader::StringReader;
use crate::{Player, PlayerExtension};
use std::collections::HashMap;

pub type CommandFunction<P> = Box<dyn for<'a> Fn(&'a mut StringReader<'a>, &mut Player<P>) -> anyhow::Result<()>>;

pub struct Command<P: PlayerExtension> {
    pub literal: &'static str,
    pub callback: CommandFunction<P>,
}

// currently doesn't branch (its just root node and arguments), as for now there is zero reason to
pub struct CommandDispatcher<P: PlayerExtension> {
    commands: HashMap<&'static str, CommandFunction<P>>
}

impl<P: PlayerExtension> CommandDispatcher<P> {

    pub fn new() -> Self {
        Self {
            commands: Default::default(),
        }
    }

    pub fn register_command(&mut self, command: Command<P>) {
        let Command { literal, callback } = command;
        self.commands.insert(literal, callback);
    }

    pub fn dispatch(&self, player: &mut Player<P>, str: &str) {
        let mut reader = StringReader::new(str);
        let literal = reader.read_word();

        if let Some(command) = self.commands.get(literal) {
            if command(&mut reader, player).is_err() {
                player.send_message("Invalid arguments.")
            }
        } else {
            player.send_message("Invalid command.")
        }
    }
}

#[macro_export]
macro_rules! command {
    ($lit:expr, | $first:ident : $first_ty:ty $(, $rest:ident : $ty:ty )* | $body:block) => {{
        server::commands::command::Command {
            literal: concat!("/", $lit),
            callback: Box::new(|reader: &mut server::commands::string_reader::StringReader<'_>, $first: $first_ty| -> anyhow::Result<()> {
                use server::commands::command_parse::CommandParse;
                $(
                    let mut $rest: $ty = CommandParse::parse(reader)?;
                )*
                $body
                Ok(())
            })
        }
    }};
}