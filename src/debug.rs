use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use chicken::states::{
    AppScope, ClientStatus, HostNewGameMenuScreen, HostSavedGameMenuScreen, JoinGameMenuScreen,
    MainMenuContext, MultiplayerSetup, NewGameMenuScreen, SavedGameMenuScreen, ServerVisibility,
    SessionState, SessionType, SingleplayerSetup, SingleplayerStatus,
};

pub struct DebugStatePlugin;

impl Plugin for DebugStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, ui_debug_state_overlay);
    }
}

fn ui_debug_state_overlay(
    mut egui: EguiContexts,
    // App Scope
    app_scope: Option<Res<State<AppScope>>>,
    // Menu States
    menu_context: Option<Res<State<MainMenuContext>>>,
    mp_setup: Option<Res<State<MultiplayerSetup>>>,
    sp_setup: Option<Res<State<SingleplayerSetup>>>,
    // Sub-menus (Multiplayer)
    host_new_game_screen: Option<Res<State<HostNewGameMenuScreen>>>,
    host_saved_game_screen: Option<Res<State<HostSavedGameMenuScreen>>>,
    join_game_screen: Option<Res<State<JoinGameMenuScreen>>>,
    // Sub-menus (Singleplayer)
    new_game_screen: Option<Res<State<NewGameMenuScreen>>>,
    saved_game_screen: Option<Res<State<SavedGameMenuScreen>>>,
    // Session States
    session_type: Option<Res<State<SessionType>>>,
    session_state: Option<Res<State<SessionState>>>,
    // Logic States
    client_status: Option<Res<State<ClientStatus>>>,
    sp_status: Option<Res<State<SingleplayerStatus>>>,
    server_visibility: Option<Res<State<ServerVisibility>>>,
) {
    let Ok(ctx) = egui.ctx_mut() else {
        return;
    };

    egui::Window::new("Debug: States")
        .default_open(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Global Scope:");
                if let Some(s) = &app_scope {
                    ui.label(format!("{:?}", s.get()));
                } else {
                    ui.label("None");
                }
            });

            ui.separator();

            ui.collapsing("Menu Context", |ui| {
                if let Some(s) = &menu_context {
                    ui.label(format!("Main: {:?}", s.get()));
                }
                if let Some(s) = &mp_setup {
                    ui.label(format!("Multiplayer: {:?}", s.get()));
                    ui.indent("mp_sub", |ui| {
                        if let Some(sub) = &host_new_game_screen {
                            ui.label(format!("HostNew: {:?}", sub.get()));
                        }
                        if let Some(sub) = &host_saved_game_screen {
                            ui.label(format!("HostSaved: {:?}", sub.get()));
                        }
                        if let Some(sub) = &join_game_screen {
                            ui.label(format!("Join: {:?}", sub.get()));
                        }
                    });
                }
                if let Some(s) = &sp_setup {
                    ui.label(format!("Singleplayer: {:?}", s.get()));
                    ui.indent("sp_sub", |ui| {
                        if let Some(sub) = &new_game_screen {
                            ui.label(format!("NewGame: {:?}", sub.get()));
                        }
                        if let Some(sub) = &saved_game_screen {
                            ui.label(format!("SavedGame: {:?}", sub.get()));
                        }
                    });
                }
            });

            ui.collapsing("Session & Logic", |ui| {
                if let Some(s) = &session_type {
                    ui.label(format!("Type: {:?}", s.get()));
                }
                if let Some(s) = &session_state {
                    ui.label(format!("State: {:?}", s.get()));
                }
                ui.separator();
                if let Some(s) = &client_status {
                    ui.label(format!("Client Status: {:?}", s.get()));
                }
                if let Some(s) = &sp_status {
                    ui.label(format!("Singleplayer Status: {:?}", s.get()));
                }
                if let Some(s) = &server_visibility {
                    ui.label(format!("Server Visibility: {:?}", s.get()));
                }
            });
        });
}
