use {
    bevy::prelude::*,
    bevy_egui::{EguiContexts, egui},
    chicken::{
        identity::PlayerIdentity,
        protocols::{
            CHAT_CLIENT_HISTORY_SIZE, CHAT_COMMAND_PREFIX, CHAT_MENTION_PREFIX,
            CHAT_MESSAGE_MAX_LENGTH, ChatCommandInfo, ChatPlayerInfo, ClientChat,
            ClientChatIdentity, ServerChat, ServerChatAutocomplete, ServerChatError,
            ServerChatHistoryResponse,
        },
        states::{ClientStatus, ServerVisibility, SingleplayerStatus},
    },
};

/// Plugin für das Chat-System auf Client-Seite
pub struct ChatPlugin;

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChatState>()
            .add_systems(
                Update,
                (
                    receive_chat_messages,
                    handle_chat_errors,
                    update_autocomplete_data,
                    update_error_timer,
                )
                    .run_if(
                        in_state(ServerVisibility::Public)
                            .or(in_state(SingleplayerStatus::Running)),
                    ),
            )
            .add_systems(
                Update,
                (handle_chat_input, render_chat_ui).run_if(
                    in_state(ServerVisibility::Public).or(in_state(SingleplayerStatus::Running)),
                ),
            )
            .add_systems(OnEnter(ClientStatus::Running), request_chat_history)
            .add_systems(OnEnter(SingleplayerStatus::Running), request_chat_history)
            .add_systems(OnEnter(ClientStatus::Running), send_chat_identity)
            .add_systems(OnEnter(SingleplayerStatus::Running), send_chat_identity);
    }
}

/// Ein einzelner Chat-Eintrag
#[derive(Debug, Clone)]
pub struct ChatEntry {
    pub sender_name: String,
    pub sender_steam_id: Option<u64>,
    pub text: String,
    pub timestamp: Option<u64>,
    pub is_system_message: bool,
}

impl From<ServerChat> for ChatEntry {
    fn from(msg: ServerChat) -> Self {
        Self {
            sender_name: msg.sender_name,
            sender_steam_id: msg.sender_steam_id,
            text: msg.text,
            timestamp: msg.timestamp,
            is_system_message: false,
        }
    }
}

/// State für Autocomplete-Funktionalität
#[derive(Debug, Clone)]
pub struct AutocompleteItem {
    pub display: String,
    pub replacement: String,
    pub description: Option<String>,
}

#[derive(Debug, Resource)]
pub struct AutocompleteState {
    /// Verfügbare Commands
    pub commands: Vec<ChatCommandInfo>,
    /// Verfügbare Spieler
    pub players: Vec<ChatPlayerInfo>,
    /// UI sichtbar?
    pub visible: bool,
    /// Gefilterte Items für die aktuelle Anzeige
    pub filtered_items: Vec<AutocompleteItem>,
    /// Aktuell ausgewählter Index
    pub selected_index: usize,
    /// Auslöser-Zeichen ('/' oder '@')
    pub trigger_char: Option<char>,
    /// Aktueller Filter-Text nach dem Trigger
    pub filter_text: String,
    /// Cursor-Position beim Öffnen des Autocomplete
    pub trigger_position: usize,
}

impl Default for AutocompleteState {
    fn default() -> Self {
        Self {
            commands: Vec::new(),
            players: Vec::new(),
            visible: false,
            filtered_items: Vec::new(),
            selected_index: 0,
            trigger_char: None,
            filter_text: String::new(),
            trigger_position: 0,
        }
    }
}

/// Haupt-Resource für den Chat-Status
#[derive(Resource)]
pub struct ChatState {
    /// Alle Chat-Nachrichten
    pub messages: Vec<ChatEntry>,
    /// Aktueller Eingabetext
    pub input: String,
    /// Ist der Chat geöffnet?
    pub is_open: bool,
    /// Hat der Chat Fokus?
    pub has_focus: bool,
    /// Ist die History geladen?
    pub history_loaded: bool,
    /// Aktuelle Fehlermeldung mit Timer
    pub error_message: Option<(String, Timer)>,
    /// Autocomplete-Status
    pub autocomplete: AutocompleteState,
    /// Eigener Spielername (für Mention-Highlighting)
    pub own_player_name: String,
    /// Scroll-Position für Chat-History
    pub scroll_to_bottom: bool,
}

impl Default for ChatState {
    fn default() -> Self {
        Self {
            messages: Vec::with_capacity(CHAT_CLIENT_HISTORY_SIZE),
            input: String::new(),
            is_open: false,
            has_focus: false,
            history_loaded: false,
            error_message: None,
            autocomplete: AutocompleteState::default(),
            own_player_name: String::new(),
            scroll_to_bottom: true,
        }
    }
}

/// Empfängt Chat-Nachrichten und History-Responses vom Server
fn receive_chat_messages(
    mut chat_state: ResMut<ChatState>,
    mut chat_events: MessageReader<ServerChat>,
    mut history_events: MessageReader<ServerChatHistoryResponse>,
) {
    // Normale Chat-Nachrichten empfangen
    for msg in chat_events.read() {
        chat_state.messages.push(ChatEntry::from(msg.clone()));

        // Begrenze auf maximale Anzahl
        if chat_state.messages.len() > CHAT_CLIENT_HISTORY_SIZE {
            chat_state.messages.remove(0);
        }

        chat_state.scroll_to_bottom = true;
    }

    // History-Response empfangen
    for response in history_events.read() {
        for msg in &response.history {
            chat_state.messages.push(ChatEntry::from(msg.clone()));
        }
        chat_state.history_loaded = true;
        chat_state.scroll_to_bottom = true;
    }
}

/// Handler für ServerChatError-Nachrichten
fn handle_chat_errors(
    mut chat_state: ResMut<ChatState>,
    mut error_events: MessageReader<ServerChatError>,
) {
    for error in error_events.read() {
        // Erstelle einen Timer für 5 Sekunden Anzeige
        let timer = Timer::from_seconds(5.0, TimerMode::Once);

        chat_state.error_message = Some((
            format!(
                "[{}] {}",
                format_error_type(&error.error_type),
                error.message
            ),
            timer,
        ));
    }
}

/// Formatiert den Error-Typ für die Anzeige
fn format_error_type(error_type: &chicken::protocols::ChatErrorType) -> &'static str {
    use chicken::protocols::ChatErrorType;
    match error_type {
        ChatErrorType::MessageTooLong => "Zu lang",
        ChatErrorType::EmptyMessage => "Leere Nachricht",
        ChatErrorType::RateLimited => "Rate Limit",
        ChatErrorType::UnknownCommand => "Unbekannter Befehl",
    }
}

/// Aktualisiert den Error-Timer
fn update_error_timer(mut chat_state: ResMut<ChatState>, time: Res<Time>) {
    if let Some((_, ref mut timer)) = chat_state.error_message {
        timer.tick(time.delta());
        if timer.just_finished() {
            chat_state.error_message = None;
        }
    }
}

/// Handler für ServerChatAutocomplete-Nachrichten
fn update_autocomplete_data(
    mut chat_state: ResMut<ChatState>,
    mut autocomplete_events: MessageReader<ServerChatAutocomplete>,
) {
    for data in autocomplete_events.read() {
        chat_state.autocomplete.commands = data.commands.clone();
        chat_state.autocomplete.players = data.players.clone();
        // Teams sind noch nicht implementiert
    }
}

/// Verarbeitet Chat-Eingabe (Tasten, Senden, etc.)
fn handle_chat_input(
    mut chat_state: ResMut<ChatState>,
    keys: Res<ButtonInput<KeyCode>>,
    mut chat_writer: MessageWriter<ClientChat>,
) {
    // Chat mit Enter öffnen/schließen (oder T)
    if !chat_state.is_open {
        if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::KeyT) {
            chat_state.is_open = true;
            chat_state.has_focus = true;
        }
        return;
    }

    // Senden mit Enter
    if keys.just_pressed(KeyCode::Enter) && chat_state.has_focus {
        if !chat_state.input.trim().is_empty() {
            // Client-seitige Validierung
            if chat_state.input.len() <= CHAT_MESSAGE_MAX_LENGTH {
                let message = ClientChat {
                    text: chat_state.input.clone(),
                };
                chat_writer.write(message);
                chat_state.input.clear();
            }
        } else {
            // Leere Eingabe = Chat schließen
            chat_state.is_open = false;
            chat_state.has_focus = false;
        }
    }

    // Chat mit Escape schließen
    if keys.just_pressed(KeyCode::Escape) && chat_state.is_open {
        chat_state.is_open = false;
        chat_state.has_focus = false;
        chat_state.autocomplete.visible = false;
        chat_state.input.clear();
    }

    // Autocomplete Navigation
    if chat_state.autocomplete.visible {
        if keys.just_pressed(KeyCode::ArrowDown) {
            if !chat_state.autocomplete.filtered_items.is_empty() {
                chat_state.autocomplete.selected_index = (chat_state.autocomplete.selected_index
                    + 1)
                    % chat_state.autocomplete.filtered_items.len();
            }
        }

        if keys.just_pressed(KeyCode::ArrowUp) {
            if !chat_state.autocomplete.filtered_items.is_empty() {
                let len = chat_state.autocomplete.filtered_items.len();
                chat_state.autocomplete.selected_index =
                    (chat_state.autocomplete.selected_index + len - 1) % len;
            }
        }

        if keys.just_pressed(KeyCode::Tab) || keys.just_pressed(KeyCode::Enter) {
            // Autocomplete-Auswahl übernehmen
            if let Some(item) = chat_state
                .autocomplete
                .filtered_items
                .get(chat_state.autocomplete.selected_index)
                .cloned()
            {
                apply_autocomplete(&mut chat_state, &item);
            }
        }
    }
}

/// Wendet eine Autocomplete-Auswahl auf den Input an
fn apply_autocomplete(chat_state: &mut ChatState, item: &AutocompleteItem) {
    let trigger_pos = chat_state.autocomplete.trigger_position;

    // Ersetze den Text vom Trigger bis zum Ende
    let before_trigger = &chat_state.input[..trigger_pos.saturating_sub(1)];
    let new_text = format!("{}{} ", before_trigger, item.replacement);

    chat_state.input = new_text;
    chat_state.autocomplete.visible = false;
    chat_state.autocomplete.filter_text.clear();
}

fn request_chat_history(
    mut client_history_writer: MessageWriter<chicken::protocols::ClientChatHistoryRequest>,
    mut chat_state: ResMut<ChatState>,
) {
    chat_state.history_loaded = false;
    client_history_writer.write(chicken::protocols::ClientChatHistoryRequest);
}

fn send_chat_identity(
    mut identity_writer: MessageWriter<ClientChatIdentity>,
    identity: Res<PlayerIdentity>,
) {
    identity_writer.write(ClientChatIdentity {
        name: identity.display_name.clone(),
        steam_id: identity.steam_id.clone(),
    });
}

/// Rendert die Chat-UI mit egui
pub fn render_chat_ui(
    mut contexts: EguiContexts,
    mut chat_state: ResMut<ChatState>,
    mut identity_writer: MessageWriter<ClientChatIdentity>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    // Sende Identity beim ersten Mal
    if chat_state.own_player_name.is_empty() {
        chat_state.own_player_name = "Player".to_string();
        identity_writer.write(ClientChatIdentity {
            name: chat_state.own_player_name.clone(),
            steam_id: None,
        });
    }

    // Chat-Fenster
    if chat_state.is_open {
        egui::Window::new("Chat")
            .anchor(egui::Align2::LEFT_BOTTOM, egui::Vec2::new(10.0, -10.0))
            .default_size([400.0, 300.0])
            .resizable(true)
            .collapsible(false)
            .title_bar(false)
            .frame(
                egui::Frame::window(&ctx.style())
                    .fill(egui::Color32::from_rgba_premultiplied(0, 0, 0, 200))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(100))),
            )
            .show(ctx, |ui| {
                // Fehlermeldung anzeigen (rot)
                if let Some((ref error_msg, _)) = chat_state.error_message {
                    ui.colored_label(egui::Color32::from_rgb(255, 100, 100), error_msg);
                    ui.separator();
                }

                // Chat-Verlauf
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .stick_to_bottom(chat_state.scroll_to_bottom)
                    .show(ui, |ui| {
                        for entry in &chat_state.messages {
                            render_chat_message(ui, entry, &chat_state.own_player_name);
                        }
                    });

                ui.separator();

                // Eingabefeld mit Zeichen-Zähler
                ui.horizontal(|ui| {
                    let text_edit = egui::TextEdit::singleline(&mut chat_state.input)
                        .hint_text("Nachricht eingeben...")
                        .desired_width(320.0)
                        .margin(egui::vec2(8.0, 6.0));

                    let response = ui.add(text_edit);

                    if chat_state.has_focus {
                        response.request_focus();
                    }

                    // Zeichen-Zähler
                    let char_count = chat_state.input.len();
                    let color = if char_count > CHAT_MESSAGE_MAX_LENGTH {
                        egui::Color32::RED
                    } else if char_count > CHAT_MESSAGE_MAX_LENGTH * 3 / 4 {
                        egui::Color32::YELLOW
                    } else {
                        egui::Color32::GRAY
                    };

                    ui.label(
                        egui::RichText::new(format!("{}/{}", char_count, CHAT_MESSAGE_MAX_LENGTH))
                            .color(color)
                            .monospace(),
                    );
                });

                // Autocomplete-UI
                update_autocomplete_ui(&mut chat_state);

                if chat_state.autocomplete.visible {
                    render_autocomplete_popup(ui, &mut chat_state);
                }
            });
    } else {
        // Kleine Hinweisanzeige wenn Chat geschlossen ist
        egui::Area::new("chat_hint".into())
            .anchor(egui::Align2::LEFT_BOTTOM, egui::Vec2::new(10.0, -10.0))
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new("Drücke ENTER oder T zum Chatten")
                        .color(egui::Color32::from_rgba_premultiplied(200, 200, 200, 150))
                        .small(),
                );
            });
    }
}

/// Rendert eine einzelne Chat-Nachricht mit Highlighting
fn render_chat_message(ui: &mut egui::Ui, entry: &ChatEntry, own_name: &str) {
    if entry.is_system_message {
        ui.colored_label(
            egui::Color32::from_rgb(200, 200, 200),
            format!("[System] {}", entry.text),
        );
        return;
    }

    ui.horizontal_wrapped(|ui| {
        // Sender-Name
        ui.colored_label(
            egui::Color32::from_rgb(100, 200, 255),
            format!("{}: ", entry.sender_name),
        );

        // Nachrichtentext mit Highlighting
        render_highlighted_text(ui, &entry.text, own_name);
    });
}

/// Rendert Text mit @mentions und /commands hervorgehoben
fn render_highlighted_text(ui: &mut egui::Ui, text: &str, own_name: &str) {
    let own_mention = format!("@{}", own_name);

    // Splitte den Text in Tokens
    for word in text.split_whitespace() {
        if word.starts_with(CHAT_MENTION_PREFIX) {
            // Mention gefunden
            let is_own_mention = word == own_mention;
            let color = if is_own_mention {
                egui::Color32::from_rgb(255, 200, 50) // Hellorange für eigene Mentions
            } else {
                egui::Color32::from_rgb(255, 165, 0) // Orange für andere Mentions
            };

            if is_own_mention {
                ui.colored_label(color, egui::RichText::new(word).strong());
            } else {
                ui.colored_label(color, word);
            }
        } else if word.starts_with(CHAT_COMMAND_PREFIX) {
            // Command gefunden
            ui.colored_label(
                egui::Color32::from_rgb(100, 255, 100), // Grün für Commands
                word,
            );
        } else {
            // Normaler Text
            ui.label(word);
        }
        ui.label(" ");
    }
}

/// Aktualisiert den Autocomplete-Status basierend auf der aktuellen Eingabe
fn update_autocomplete_ui(chat_state: &mut ChatState) {
    let input = &chat_state.input;

    // Prüfe ob wir im Autocomplete-Modus sein sollten
    let last_trigger_pos = input.rfind(|c| c == CHAT_COMMAND_PREFIX || c == CHAT_MENTION_PREFIX);

    if let Some(pos) = last_trigger_pos {
        // Prüfe ob es ein neuer Trigger ist (nach Leerzeichen oder am Anfang)
        let is_new_trigger = pos == 0 || input.chars().nth(pos.saturating_sub(1)) == Some(' ');

        if is_new_trigger {
            let trigger_char = input.chars().nth(pos).unwrap_or('/');
            let after_trigger = &input[pos + 1..];

            // Nur wenn kein Leerzeichen nach dem Trigger (wir sind noch im "Wort")
            if !after_trigger.contains(' ') {
                chat_state.autocomplete.visible = true;
                chat_state.autocomplete.trigger_char = Some(trigger_char);
                chat_state.autocomplete.trigger_position = pos + 1;
                chat_state.autocomplete.filter_text = after_trigger.to_lowercase();

                // Filtere Items basierend auf Trigger und Filter-Text
                chat_state.autocomplete.filtered_items = match trigger_char {
                    CHAT_COMMAND_PREFIX => filter_commands(
                        &chat_state.autocomplete.commands,
                        &chat_state.autocomplete.filter_text,
                    ),
                    CHAT_MENTION_PREFIX => filter_players(
                        &chat_state.autocomplete.players,
                        &chat_state.autocomplete.filter_text,
                    ),
                    _ => Vec::new(),
                };

                // Reset selection wenn sich Filter ändert
                chat_state.autocomplete.selected_index = 0;
                return;
            }
        }
    }

    // Kein Autocomplete-Trigger gefunden
    chat_state.autocomplete.visible = false;
    chat_state.autocomplete.filter_text.clear();
}

/// Filtert Commands basierend auf dem Filter-Text
fn filter_commands(commands: &[ChatCommandInfo], filter: &str) -> Vec<AutocompleteItem> {
    commands
        .iter()
        .filter(|cmd| cmd.command.to_lowercase().contains(filter))
        .map(|cmd| AutocompleteItem {
            display: format!("/{} - {}", cmd.command, cmd.description),
            replacement: format!("/{}", cmd.command),
            description: Some(cmd.usage.clone()),
        })
        .collect()
}

/// Filtert Spieler basierend auf dem Filter-Text
fn filter_players(players: &[ChatPlayerInfo], filter: &str) -> Vec<AutocompleteItem> {
    players
        .iter()
        .filter(|player| player.name.to_lowercase().contains(filter))
        .map(|player| AutocompleteItem {
            display: format!("@{}", player.name),
            replacement: format!("@{}", player.name),
            description: player.steam_id.map(|id| format!("SteamID: {}", id)),
        })
        .collect()
}

/// Rendert das Autocomplete-Popup
fn render_autocomplete_popup(ui: &mut egui::Ui, chat_state: &mut ChatState) {
    if chat_state.autocomplete.filtered_items.is_empty() {
        return;
    }

    egui::Frame::popup(ui.style())
        .fill(egui::Color32::from_rgba_premultiplied(30, 30, 30, 240))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(80)))
        .show(ui, |ui| {
            ui.set_max_height(150.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                let selected_index = chat_state.autocomplete.selected_index;
                let items: Vec<_> = chat_state.autocomplete.filtered_items.clone();

                for (idx, item) in items.iter().enumerate() {
                    let is_selected = idx == selected_index;

                    let text = if let Some(ref desc) = item.description {
                        format!("{} - {}", item.display, desc)
                    } else {
                        item.display.clone()
                    };

                    let response = if is_selected {
                        ui.add(egui::Label::new(
                            egui::RichText::new(&text)
                                .strong()
                                .color(egui::Color32::WHITE)
                                .background_color(egui::Color32::from_rgb(50, 100, 150)),
                        ))
                    } else {
                        ui.label(text)
                    };

                    if response.clicked() {
                        apply_autocomplete(chat_state, item);
                    }
                }
            });

            ui.separator();
            ui.label(
                egui::RichText::new("↑↓ Navigation  |  Tab/Enter Auswahl  |  ESC Schließen")
                    .small()
                    .color(egui::Color32::GRAY),
            );
        });
}
