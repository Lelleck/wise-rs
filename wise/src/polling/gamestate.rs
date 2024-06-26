use rcon::{connection::RconConnection, parsing::gamestate::GameState};
use serde::Serialize;
use tokio::time::sleep;
use tracing::{debug, error, instrument, warn};

use crate::event::RconEvent;

use super::{
    utils::{detect, fetch},
    PollerContext,
};

#[derive(Debug, Clone, Serialize)]
pub enum GameStateChanges {
    AlliedPlayers { old: u64, new: u64 },
    AxisPlayers { old: u64, new: u64 },
    AlliedScore { old: u64, new: u64 },
    AxisScore { old: u64, new: u64 },
    Map { old: String, new: String },
    NextMap { old: String, new: String },
}

#[instrument(level = "debug", skip_all, fields(poller_id = ctx.id))]
pub async fn poll_gamestate(mut ctx: PollerContext) {
    let connection = RconConnection::new(&ctx.config.rcon).await;
    if let Err(e) = connection {
        warn!("Failed to establish connection: {}", e);
        return;
    }
    let mut connection = connection.unwrap();
    let mut previous = None;

    loop {
        sleep(ctx.config.polling.wait_ms).await;

        let fetch_gamestate = connection.fetch_gamestate().await;
        let new_gamestate = fetch(&mut connection, fetch_gamestate, &ctx.config).await;
        if let Err((recoverable, e)) = new_gamestate {
            if !recoverable {
                error!("Unrecoverable error: {}", e);
                return;
            }
            
            warn!("Recoverable error: {}", e);
            continue;
        }
        let current = new_gamestate.unwrap();

        if previous.is_none() {
            debug!("Started polling with: {:?}", current);
            ctx.tx.send_rcon(RconEvent::Game {
                changes: vec![],
                new_state: current.clone(),
            });
            previous = Some(current);
            continue;
        }
        let old = previous.clone().unwrap();

        let changes = detect_changes(&old, &current);
        if changes.is_empty() {
            continue;
        }

        debug!("Detected changes {:?}", changes);
        ctx.tx.send_rcon(RconEvent::Game {
            changes,
            new_state: current.clone(),
        });
        previous = Some(current);
    }
}

fn detect_changes(old: &GameState, new: &GameState) -> Vec<GameStateChanges> {
    if *old == *new {
        return vec![];
    }

    let mut changes = vec![];

    detect(
        &mut changes,
        &old.allied_players,
        &new.allied_players,
        GameStateChanges::AlliedPlayers {
            old: old.allied_players,
            new: new.allied_players,
        },
    );

    detect(
        &mut changes,
        &old.axis_players,
        &new.axis_players,
        GameStateChanges::AxisPlayers {
            old: old.axis_players,
            new: new.axis_players,
        },
    );

    detect(
        &mut changes,
        &old.allied_score,
        &new.allied_score,
        GameStateChanges::AlliedScore {
            old: old.allied_score,
            new: new.allied_score,
        },
    );

    detect(
        &mut changes,
        &old.axis_score,
        &new.axis_score,
        GameStateChanges::AxisScore {
            old: old.axis_score,
            new: new.axis_score,
        },
    );

    detect(
        &mut changes,
        &old.map,
        &new.map,
        GameStateChanges::Map {
            old: old.map.clone(),
            new: new.map.clone(),
        },
    );

    detect(
        &mut changes,
        &old.next_map,
        &new.next_map,
        GameStateChanges::NextMap {
            old: old.next_map.clone(),
            new: new.next_map.clone(),
        },
    );

    changes
}
