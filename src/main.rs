use bevy::app::AppExit;
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use chicken::network::client::{
    ClientTarget, DiscoveredServers, DiscoveryControl, SetClientTarget,
};
use chicken::states::events::app::SetAppScope;
use chicken::states::events::menu::multiplayer::{
    SetJoinGame, SetMultiplayerMenu, SetNewHostGame, SetSavedHostGame,
};
use chicken::states::events::menu::settings::SetSettingsMenu;
use chicken::states::events::menu::singleplayer::{
    SetSingleplayerMenu, SetSingleplayerNewGame, SetSingleplayerSavedGame,
};
use chicken::states::events::menu::wiki::SetWikiMenu;
use chicken::states::events::session::{SetGoingPrivateStep, SetGoingPublicStep, SetPauseMenu};
use chicken::states::states::session::{ClientConnectionStatus, ServerStatus};
use chicken::states::states::menu::multiplayer::{HostNewGameMenuScreen, HostSavedGameMenuScreen};
use chicken::{
    states::states::{
        app::AppScope,
        menu::{
            main::MainMenuScreen, multiplayer::MultiplayerMenuScreen,
            singleplayer::{NewGameMenuScreen, SingleplayerMenuScreen},
        },
        session::{ServerShutdownStep, ServerVisibility, SessionState, SessionType},
    },
    // steam::SteamworksPlugin,
};
use client::{FOSClientPlugin, chat, debug::DebugStatePlugin};

fn main() -> AppExit {
    // let steam_client =
    //     SteamworksPlugin::init_app(client::STEAM_APP_ID).expect("failed to initialize steam");

    App::new()
        .add_plugins((
            // steam_client,
            DefaultPlugins,
            EguiPlugin::default(),
            WorldInspectorPlugin::new(),
            FOSClientPlugin,
            DebugStatePlugin,
        ))
        .add_systems(Startup, setup_camera_system)
        .add_systems(
            EguiPrimaryContextPass,
            ui_menu_system.run_if(in_state(AppScope::Menu)),
        )
        .add_systems(
            EguiPrimaryContextPass,
            ui_singleplayer_system
                .run_if(in_state(AppScope::Session))
                .run_if(in_state(SessionType::Singleplayer)),
        )
        .add_systems(
            EguiPrimaryContextPass,
            ui_client_system
                .run_if(in_state(AppScope::Session))
                .run_if(in_state(SessionType::Client)),
        )
        .add_systems(
            EguiPrimaryContextPass,
            ui_game_menu
                .run_if(in_state(AppScope::Session))
                .run_if(in_state(SessionState::Paused)),
        )
        .add_systems(
            EguiPrimaryContextPass,
            chat::render_chat_ui.run_if(in_state(SessionState::Active)),
        )
        .run()
}

fn setup_camera_system(mut commands: Commands) {
    commands.spawn(Camera2d);
}

#[derive(SystemParam)]
struct MenuUiParams<'w, 's> {
    commands: Commands<'w, 's>,
    egui: EguiContexts<'w, 's>,
    app_state: Res<'w, State<AppScope>>,
    menu_state: Res<'w, State<MainMenuScreen>>,
    singleplayer_menu_state: Option<Res<'w, State<SingleplayerMenuScreen>>>,
    new_game_menu_state: Option<Res<'w, State<NewGameMenuScreen>>>,
    host_new_game_menu_state: Option<Res<'w, State<HostNewGameMenuScreen>>>,
    host_saved_game_menu_state: Option<Res<'w, State<HostSavedGameMenuScreen>>>,
    multiplayer_menu_state: Option<Res<'w, State<MultiplayerMenuScreen>>>,
    discovered_servers: Option<Res<'w, DiscoveredServers>>,
    discovery_control: Option<ResMut<'w, DiscoveryControl>>,
    client_target: Option<ResMut<'w, ClientTarget>>,
}

struct MenuActions<'w, 's> {
    commands: Commands<'w, 's>,
}

// --- UI SYSTEM ---

fn ui_singleplayer_system(
    mut _commands: Commands,
    mut egui: EguiContexts,
    app_state: Res<State<AppScope>>,
    game_mode_state: Res<State<SessionType>>,
    singleplayer_state: Res<State<ServerStatus>>,
    session_state: Option<Res<State<SessionState>>>,
    shutdown_step: Option<Res<State<ServerShutdownStep>>>,
) -> Result<(), bevy::prelude::BevyError> {
    egui::Window::new("APP Game - Singleplayer").show(egui.ctx_mut()?, |ui| {
        ui.vertical_centered_justified(|ui| {
            if *app_state.get() == AppScope::Session
                && *game_mode_state.get() == SessionType::Singleplayer
            {
                ui.label(format!(
                    "States: \n AppScope: {:?}\nSessionType: {:?}\nSingleplayer: {:?}\n Lifecycle: {:?}\n ShutdownStep: {:?}",
                    app_state, game_mode_state, singleplayer_state, session_state, shutdown_step
                ));
                match *singleplayer_state.get() {
                    ServerStatus::Running => {
                        ui.label("Singleplayer is running");
                        ui.separator();
                    }
                    _ => {
                        ui.label("Singleplayer is not running");
                    }
                }
            }
        });
    });
    Ok(())
}

fn ui_client_system(
    mut _commands: Commands,
    mut egui: EguiContexts,
    app_state: Res<State<AppScope>>,
    game_mode_state: Res<State<SessionType>>,
    client_state: Res<State<ClientConnectionStatus>>,
) -> Result<(), bevy::prelude::BevyError> {
    egui::Window::new("APP Game - Client").show(egui.ctx_mut()?, |ui| {
        ui.vertical_centered_justified(|ui| {
            if *app_state.get() == AppScope::Session
                && *game_mode_state.get() == SessionType::Client
            {
                ui.label(format!(
                    "States: \n AppScope: {:?}\nGameMode: {:?}\nClientState: {:?}\n",
                    app_state, game_mode_state, client_state
                ));
            }
        });
    });
    Ok(())
}

fn ui_game_menu(
    mut commands: Commands,
    mut egui: EguiContexts,
    app_state: Res<State<AppScope>>,
    game_mode_state: Res<State<SessionType>>,
    session_state: Res<State<SessionState>>,
    server_visibility: Option<Res<State<ServerVisibility>>>,
) -> Result<(), bevy::prelude::BevyError> {
    egui::Window::new("APP Game Menu").show(egui.ctx_mut()?, |ui| {
        ui.vertical_centered_justified(|ui| {
            if *app_state.get() == AppScope::Session && *session_state.get() == SessionState::Paused
            {
                ui.label("Game Menu");
                ui.button("Resume").clicked().then(|| {
                    commands.trigger(SetPauseMenu::Resume);
                });
                match *game_mode_state.get() {
                    SessionType::Singleplayer => {
                        if let Some(server_visibility) = server_visibility {
                            match *server_visibility.get() {
                                ServerVisibility::Private => {
                                    ui.button("Open to LAN ").clicked().then(|| {
                                        commands.trigger(SetGoingPublicStep::Start);
                                    });
                                }
                                ServerVisibility::Public => {
                                    if let Some(ip) =
                                        chicken::network::server::networking::address::get_local_ip(
                                        )
                                    {
                                        ui.label(format!("Server IP: {}", ip));
                                    }
                                    ui.button("Close to LAN").clicked().then(|| {
                                        commands.trigger(SetGoingPrivateStep::Start);
                                    });
                                }
                                _ => {}
                            };
                        }
                    }
                    SessionType::Client => {}
                    SessionType::None => {}
                };
                ui.button("Exit").clicked().then(|| {
                    commands.trigger(SetPauseMenu::Exit);
                });
            }
        });
    });
    Ok(())
}

fn ui_menu_system(mut params: MenuUiParams) -> Result<(), BevyError> {
    // 1. Get ctx (only &mut-borrow on params.egui, no move)
    let ctx = params.egui.ctx_mut()?;

    // 2. Pack everything you need into local references
    let app_state = &params.app_state;
    let menu_state = &params.menu_state;
    let single = params.singleplayer_menu_state.as_deref();
    let new_game = params.new_game_menu_state.as_deref();
    let host_new_game = params.host_new_game_menu_state.as_deref();
    let host_saved_game = params.host_saved_game_menu_state.as_deref();
    let multi = params.multiplayer_menu_state.as_deref();
    let discovered = params.discovered_servers.as_deref();
    let discovery_control = params.discovery_control.as_deref_mut();
    let client_target = params.client_target.as_deref_mut();

    // 3. Build mutable "Action" bundle for Commands + Exit
    let mut actions = MenuActions {
        commands: params.commands,
    };

    egui::Window::new("APP Menu").show(ctx, |ui| {
        ui.vertical_centered_justified(|ui| {
            if app_state.get() != &AppScope::Menu {
                return;
            }

            match menu_state.get() {
                MainMenuScreen::Overview => render_menu_main(ui, &mut actions),
                MainMenuScreen::Singleplayer => render_singleplayer_menu(ui, &mut actions, single, new_game),
                MainMenuScreen::Multiplayer => render_multiplayer_menu(
                    ui,
                    &mut actions,
                    multi,
                    host_new_game,
                    host_saved_game,
                    discovered,
                    discovery_control,
                    client_target,
                ),
                MainMenuScreen::Wiki => render_menu_wiki(ui, &mut actions),
                MainMenuScreen::Settings => render_menu_settings(ui, &mut actions),
            }
        });
    });

    Ok(())
}

fn render_menu_main(ui: &mut egui::Ui, actions: &mut MenuActions) {
    if ui.button("Singleplayer").clicked() {
        actions.commands.trigger(SetSingleplayerMenu::Overview);
    }
    if ui.button("Multiplayer").clicked() {
        actions.commands.trigger(SetMultiplayerMenu::Overview);
    }
    if ui.button("Wiki").clicked() {
        actions.commands.trigger(SetWikiMenu::Overview);
    }
    if ui.button("Settings").clicked() {
        actions.commands.trigger(SetSettingsMenu::Overview);
    }
    if ui.button("Quit").clicked() {
        actions.commands.trigger(SetAppScope::Exit);
    }
}

fn render_singleplayer_menu(
    ui: &mut egui::Ui,
    actions: &mut MenuActions,
    state: Option<&State<SingleplayerMenuScreen>>,
    new_game: Option<&State<NewGameMenuScreen>>,
) {
    ui.vertical_centered_justified(|ui| {
        let Some(single) = state else {
            return;
        };

        match single.get() {
            SingleplayerMenuScreen::Overview => {
                render_singleplayer_overview(ui, actions);
            }
            SingleplayerMenuScreen::NewGame => {
                render_singleplayer_new_game(ui, actions, new_game);
            }
            SingleplayerMenuScreen::LoadGame => {
                render_singleplayer_load_game(ui, actions);
            }
        }
    });
}

fn render_singleplayer_overview(ui: &mut egui::Ui, actions: &mut MenuActions) {
    if ui.button("New Game").clicked() {
        actions.commands.trigger(SetSingleplayerMenu::NewGame);
    }
    if ui.button("Load Game").clicked() {
        actions.commands.trigger(SetSingleplayerMenu::LoadGame);
    }
    if ui.button("Back").clicked() {
        actions.commands.trigger(SetSingleplayerMenu::Back);
    }
}

fn render_singleplayer_new_game(
    ui: &mut egui::Ui,
    actions: &mut MenuActions,
    state: Option<&State<NewGameMenuScreen>>,
) {
    let Some(step) = state else {
        return;
    };

    match step.get() {
        NewGameMenuScreen::ConfigPlayer => {
            ui.label("Step 1/3: Configure Player");
            if ui.button("Next →").clicked() {
                actions.commands.trigger(SetSingleplayerNewGame::Next);
            }
        }
        NewGameMenuScreen::ConfigWorld => {
            ui.label("Step 2/3: Configure World");
            if ui.button("← Previous").clicked() {
                actions.commands.trigger(SetSingleplayerNewGame::Previous);
            }
            if ui.button("Next →").clicked() {
                actions.commands.trigger(SetSingleplayerNewGame::Next);
            }
        }
        NewGameMenuScreen::ConfigSave => {
            ui.label("Step 3/3: Configure Save");
            if ui.button("← Previous").clicked() {
                actions.commands.trigger(SetSingleplayerNewGame::Previous);
            }
            if ui.button("Start Game").clicked() {
                actions.commands.trigger(SetSingleplayerNewGame::Confirm);
            }
        }
    }

    if ui.button("Cancel").clicked() {
        actions.commands.trigger(SetSingleplayerNewGame::Cancel);
    }
}

fn render_singleplayer_load_game(ui: &mut egui::Ui, actions: &mut MenuActions) {
    if ui.button("Load").clicked() {
        actions.commands.trigger(SetSingleplayerSavedGame::Confirm);
    }
    if ui.button("Back").clicked() {
        actions.commands.trigger(SetSingleplayerSavedGame::Cancel);
    }
}

fn render_multiplayer_menu(
    ui: &mut egui::Ui,
    actions: &mut MenuActions,
    state: Option<&State<MultiplayerMenuScreen>>,
    host_new_game: Option<&State<HostNewGameMenuScreen>>,
    host_saved_game: Option<&State<HostSavedGameMenuScreen>>,
    discovered_servers: Option<&DiscoveredServers>,
    discovery_control: Option<&mut DiscoveryControl>,
    client_target: Option<&mut ClientTarget>,
) {
    ui.vertical_centered_justified(|ui| {
        let Some(multi) = state else {
            return;
        };

        match multi.get() {
            MultiplayerMenuScreen::Overview => {
                render_multiplayer_overview(ui, actions);
            }
            MultiplayerMenuScreen::HostNewGame => {
                render_multiplayer_host_new(ui, actions, host_new_game);
            }
            MultiplayerMenuScreen::HostSavedGame => {
                render_multiplayer_host_saved(ui, actions, host_saved_game);
            }
            MultiplayerMenuScreen::JoinGame => {
                render_multiplayer_join_game(
                    ui,
                    actions,
                    discovered_servers,
                    discovery_control,
                    client_target,
                );
            }
        }
    });
}

fn render_multiplayer_overview(ui: &mut egui::Ui, actions: &mut MenuActions) {
    if ui.button("Host new Game").clicked() {
        actions.commands.trigger(SetMultiplayerMenu::HostNewGame);
    }

    if ui.button("Host saved Game").clicked() {
        actions.commands.trigger(SetMultiplayerMenu::HostSavedGame);
    }

    if ui.button("Join Game").clicked() {
        actions.commands.trigger(SetMultiplayerMenu::JoinGame);
    }

    if ui.button("Back").clicked() {
        actions.commands.trigger(SetMultiplayerMenu::Back);
    }
}

fn render_multiplayer_host_new(
    ui: &mut egui::Ui,
    actions: &mut MenuActions,
    state: Option<&State<HostNewGameMenuScreen>>,
) {
    let Some(step) = state else {
        return;
    };

    match step.get() {
        HostNewGameMenuScreen::ConfigServer => {
            ui.label("Step 1/3: Configure Server");
            if ui.button("Next →").clicked() {
                actions.commands.trigger(SetNewHostGame::Next);
            }
        }
        HostNewGameMenuScreen::ConfigWorld => {
            ui.label("Step 2/3: Configure World");
            if ui.button("← Previous").clicked() {
                actions.commands.trigger(SetNewHostGame::Previous);
            }
            if ui.button("Next →").clicked() {
                actions.commands.trigger(SetNewHostGame::Next);
            }
        }
        HostNewGameMenuScreen::ConfigSave => {
            ui.label("Step 3/3: Configure Save");
            if ui.button("← Previous").clicked() {
                actions.commands.trigger(SetNewHostGame::Previous);
            }
            if ui.button("Start Server").clicked() {
                actions.commands.trigger(SetNewHostGame::Confirm);
            }
        }
    }

    if ui.button("Cancel").clicked() {
        actions.commands.trigger(SetNewHostGame::Cancel);
    }
}

fn render_multiplayer_host_saved(
    ui: &mut egui::Ui,
    actions: &mut MenuActions,
    state: Option<&State<HostSavedGameMenuScreen>>,
) {
    let Some(step) = state else {
        return;
    };

    match step.get() {
        HostSavedGameMenuScreen::Overview => {
            ui.label("Step 1/2: Select Save");
            if ui.button("Next →").clicked() {
                actions.commands.trigger(SetSavedHostGame::Next);
            }
        }
        HostSavedGameMenuScreen::ConfigServer => {
            ui.label("Step 2/2: Configure Server");
            if ui.button("← Previous").clicked() {
                actions.commands.trigger(SetSavedHostGame::Previous);
            }
            if ui.button("Start Server").clicked() {
                actions.commands.trigger(SetSavedHostGame::Confirm);
            }
        }
    }

    if ui.button("Cancel").clicked() {
        actions.commands.trigger(SetSavedHostGame::Cancel);
    }
}

fn render_multiplayer_join_game(
    ui: &mut egui::Ui,
    actions: &mut MenuActions,
    discovered_servers: Option<&DiscoveredServers>,
    mut discovery_control: Option<&mut DiscoveryControl>,
    client_target: Option<&mut ClientTarget>,
) {
    ui.heading("Local Servers");

    ui.horizontal(|ui| {
        if let Some(control) = discovery_control.as_deref_mut() {
            if control.cycles_remaining > 0 {
                ui.label("Searching... ");
                ui.add(egui::Spinner::new());
            } else {
                ui.label("Scan finished.");
                if ui.button("Refresh").clicked() {
                    control.cycles_remaining = 5;
                    control.timer.reset();
                }
            }
        } else {
            ui.label("Discovery inactive");
        }
    });

    // Local Servers List
    if let Some(res) = discovered_servers {
        let servers = &res.0;
        if !servers.is_empty() {
            ui.separator();
            for server in servers {
                if ui.selectable_label(false, server.to_string()).clicked() {
                    // ✅ input instead of target
                    actions.commands.queue(SetClientTarget {
                        input: server.clone(),
                    });
                }
            }
            ui.separator();
        } else if let Some(control) = discovery_control {
            // Only show "No servers found" if scan is finished
            if control.cycles_remaining == 0 {
                ui.label("No servers found.");
            }
        }
    }

    ui.separator();

    let mut is_client_target_valid = false;
    if let Some(target) = client_target {
        ui.horizontal(|ui| {
            let response =
                ui.add(egui::TextEdit::singleline(&mut target.input).hint_text("127.0.0.1:8080"));

            if response.changed() {
                let val = target.input.clone();
                target.update_input(val);
            }

            // Display status
            ui.label(match target.is_valid {
                true => "Valid",
                false => {
                    if target.input.trim().is_empty() {
                        ""
                    } else {
                        "Invalid"
                    }
                }
            });
        });
        ui.label(format!(
            "Client Target:\nInput:{}\nIP-Address:{:?}\nPort:{}\nIs valid:{}",
            target.input, target.ip, target.port, target.is_valid
        ));

        is_client_target_valid = target.is_valid;
    }

    ui.separator();

    let join_button = ui.add_enabled(
        is_client_target_valid,
        egui::Button::new("Join Selected Game"),
    );

    if join_button.clicked() {
        actions.commands.trigger(SetJoinGame::Confirm);
    }

    ui.separator();

    if ui.button("Back").clicked() {
        actions.commands.trigger(SetJoinGame::Cancel);
    }
}

fn render_menu_wiki(ui: &mut egui::Ui, actions: &mut MenuActions) {
    ui.vertical_centered_justified(|ui| {
        if ui.button("Back").clicked() {
            actions.commands.trigger(SetWikiMenu::Back);
        }
    });
}

fn render_menu_settings(ui: &mut egui::Ui, actions: &mut MenuActions) {
    ui.vertical_centered_justified(|ui| {
        if ui.button("Back").clicked() {
            actions.commands.trigger(SetSettingsMenu::Back);
        }
    });
}
