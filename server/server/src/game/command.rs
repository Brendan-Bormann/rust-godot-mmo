use std::sync::mpsc;

use tokio::sync::oneshot::{self, Receiver};
use tracing::warn;

pub struct Command {
    pub issuer: Option<String>,
    pub packet_id: String,
    pub cmd_type: i16,
    pub arguments: Option<Vec<u8>>,

    // called by game @ process time
    pub done: oneshot::Sender<CommandResponse>,
}

impl Command {
    pub fn new(
        issuer: Option<String>,
        packet_id: String,
        cmd_type: i16,
        arguments: Option<Vec<u8>>,
    ) -> (Self, oneshot::Receiver<CommandResponse>) {
        let (tx, rx) = oneshot::channel::<CommandResponse>();
        (
            Command {
                issuer,
                packet_id,
                cmd_type,
                arguments,
                done: tx,
            },
            // command issuer listen for command result
            rx,
        )
    }

    pub fn respond(self, response: CommandResponse) {
        let _ = self.done.send(response);
    }

    pub fn respond_err_code(self, code: i16) {
        let _ = self.done.send(CommandResponse::new_err(code));
    }
}

pub struct CommandResponse {
    response: Result<Option<Vec<u8>>, i16>,
}

impl CommandResponse {
    pub fn new(response: Result<Option<Vec<u8>>, i16>) -> Self {
        CommandResponse { response }
    }

    pub fn new_err(code: i16) -> Self {
        CommandResponse {
            response: Err(code),
        }
    }
}

pub struct CommandHandlerClient {
    player_id: Option<String>,
    tx: mpsc::Sender<Command>,
}

impl CommandHandlerClient {
    pub fn new(tx: mpsc::Sender<Command>) -> Self {
        CommandHandlerClient {
            player_id: None,
            tx,
        }
    }

    pub fn set_player_id(&mut self, id: &String) {
        self.player_id = Some(id.into());
    }

    fn send_command(&mut self, command: Command) {
        self.tx.send(command).unwrap()
    }

    fn get_command_result(&mut self, rx: oneshot::Receiver<CommandResponse>) -> CommandResponse {
        match rx.blocking_recv() {
            Ok(response) => response,
            Err(e) => {
                warn!("Error in command oneshot: {}", e);
                CommandResponse::new(Err(-1))
            }
        }
    }

    pub fn create_character(
        &mut self,
        packet_id: &String,
        payload: Option<Vec<u8>>,
    ) -> Result<String, ()> {
        if self.player_id.is_some() {
            return Err(());
        }
        let (command, rx) = Command::new(None, packet_id.into(), 1, payload);

        match self.tx.send(command) {
            Ok(_) => {
                match rx.blocking_recv() {
                    Ok(cmd_resp) => {
                        match cmd_resp.response {
                            Ok(data) => {
                                return Ok(bitcode::decode::<String>(&data.unwrap()).unwrap());
                            }
                            Err(e) => {
                                warn!("Server responded with error: {}", e);
                                return Err(());
                            }
                        };
                    }
                    Err(e) => {
                        warn!("Error in command oneshot: {}", e);
                        return Err(());
                    }
                };
            }
            Err(e) => {
                warn!("Failed to send command: {}", e);
                Err(())
            }
        }
    }

    pub fn set_character_bearing(
        &mut self,
        packet_id: &String,
        payload: Option<Vec<u8>>,
    ) -> Result<Receiver<CommandResponse>, ()> {
        if self.player_id.is_none() {
            return Err(());
        }

        let (command, rx) = Command::new(self.player_id.clone(), packet_id.into(), 2, payload);

        match self.tx.send(command) {
            Ok(_) => Ok(rx),
            Err(e) => {
                warn!("Failed to send command: {}", e);
                Err(())
            }
        }
    }
}
