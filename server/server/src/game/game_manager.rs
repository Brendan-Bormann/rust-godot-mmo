use std::{collections::HashMap, thread, time::Duration};

use shared::game::{
    game_state::GameState,
    player::Player,
    vector::{Vector2, Vector3},
};
use std::sync::mpsc;
use tokio::sync::watch;
use tracing::info;

use crate::game::map::{Map, Tile};

use super::{
    command::{Command, CommandResponse},
    map,
};

pub struct GameManager {
    tick_rate: u64, // ticks per second
    state: GameState,
    state_watch_tx: watch::Sender<GameState>,
    cmd_rx: mpsc::Receiver<Command>,
    rolling_id: i16, // use self.next_id()
    map: Map,
}

impl GameManager {
    pub fn new() -> (Self, watch::Receiver<GameState>, mpsc::Sender<Command>) {
        let initial_state = GameState::new();
        let (state_tx, state_rx) = watch::channel(initial_state.clone());
        let (cmd_tx, cmd_rx) = mpsc::channel::<Command>();

        let map = Map::new("map.txt".into());

        (
            GameManager {
                tick_rate: 30,
                state: initial_state,
                state_watch_tx: state_tx,
                cmd_rx,
                rolling_id: 1,
                map,
            },
            state_rx,
            cmd_tx,
        )
    }
}

impl GameManager {
    pub fn start(&mut self) {
        info!("GameManager started");
        let mut player = Player::new("1".into(), "TheLegend27".into());
        player.input_direction = Vector2::new(1.0, 0.0);
        self.state.players.insert(player.id.clone(), player.clone());

        loop {
            self.recv_commands();
            self.sync_state();
            self.apply_input(self.delta());
            thread::sleep(Duration::from_millis(1000 / self.tick_rate));
        }
    }

    fn delta(&self) -> f64 {
        1.0 / self.tick_rate as f64
    }

    fn next_id(&mut self) -> String {
        let id = self.rolling_id;
        self.rolling_id += 1;

        id.to_string()
    }

    fn recv_commands(&mut self) {
        let cmds: Vec<_> = self.cmd_rx.try_iter().collect();

        if cmds.len() > 0 {
            // info!("GameManager batched {} commands", cmds.len());

            for cmd in cmds {
                self.process_command(cmd);
            }
        }
    }

    fn process_command(&mut self, command: Command) {
        let args = command.arguments.clone();
        let player_id = command.issuer.clone();

        match command.cmd_type {
            1 => {
                if let Some(data) = args {
                    let username: String = bitcode::decode(&data).unwrap();
                    let mut player = Player::new(self.next_id(), username);

                    player.position = Vector2 { x: 1.5, y: 1.5 };

                    self.state.players.insert(player.id.clone(), player.clone());
                    command.respond(CommandResponse::new(Ok(Some(bitcode::encode(&player.id)))));
                    return;
                }
            }
            2 => {
                if let Some(data) = args {
                    let di: Vector2 = bitcode::decode(&data).unwrap();
                    let player = self.state.players.get_mut(&player_id.unwrap()).unwrap();
                    player.input_direction = di;
                    command.respond(CommandResponse::new(Ok(None)));
                    return;
                }
            }
            _ => {}
        }

        command.respond_err_code(-1);
    }

    fn sync_state(&mut self) {
        self.state_watch_tx.send_if_modified(|old_state| {
            if *old_state == self.state {
                false
            } else {
                *old_state = self.state.clone();
                true
            }
        });
    }

    fn apply_input(&mut self, delta_time: f64) {
        self.state.players.iter_mut().for_each(|(_, player)| {
            let target_pos = player.get_target_pos(delta_time);

            let valid_move = self.map.move_to(player.position, target_pos);

            if valid_move.is_ok() {
                player.move_to(target_pos);
            } else {
                let target_pos = valid_move.unwrap_err();
                player.move_to(target_pos);
            }
        });
    }
}
