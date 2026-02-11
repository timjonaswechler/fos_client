use {
    bevy::prelude::*,
    bevy_egui::{egui, EguiContexts},
    chicken::{
        network::LocalClient,
        protocols::{
            handle_client_chat, handle_client_chat_history_request, handle_client_chat_identity,
            ClientChat, ClientChatHistoryRequest, ClientChatIdentity, ServerChat,
            ServerChatHistoryResponse,
        },
        states::{ClientStatus, ServerVisibility, SingleplayerStatus},
    },
};

pub struct ChatPlugin;

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChatState>()
            .add_systems(Update, (receive_chat_messages, handle_chat_input))
            .add_systems(
                Update,
                (
                    handle_client_chat,
                    handle_client_chat_history_request,
                    handle_client_chat_identity,
                )
                    .run_if(
                        in_state(ServerVisibility::Public)
                            .or(in_state(SingleplayerStatus::Running)),
                    ),
            )
            .add_systems(OnEnter(ClientStatus::Running), request_chat_history)
            .add_systems(OnEnter(SingleplayerStatus::Running), request_chat_history)
            .add_systems(OnEnter(ClientStatus::Running), send_chat_identity)
            .add_systems(OnEnter(SingleplayerStatus::Running), send_chat_identity);
    }
}

#[derive(Clone)]
pub struct ChatEntry {
    pub sender_name: String,
    pub sender_steam_id: Option<u64>,
    pub text: String,
}

#[derive(Resource, Default)]
pub struct ChatState {
    pub messages: Vec<ChatEntry>,
    pub input: String,
    pub is_open: bool,
    pub has_focus: bool,
    pub history_loaded: bool,
}

fn receive_chat_messages(
    mut chat_events: MessageReader<ServerChat>,
    mut chat_history_events: MessageReader<ServerChatHistoryResponse>,
    mut chat_state: ResMut<ChatState>,
) {
    for event in chat_history_events.read() {
        if !chat_state.history_loaded {
            chat_state.messages.clear();
            chat_state
                .messages
                .extend(event.history.iter().cloned().map(|entry| ChatEntry {
                    sender_name: entry.sender_name,
                    sender_steam_id: entry.sender_steam_id,
                    text: entry.text,
                }));
            chat_state.history_loaded = true;
        }
    }

    for event in chat_events.read() {
        chat_state.messages.push(ChatEntry {
            sender_name: event.sender_name.clone(),
            sender_steam_id: event.sender_steam_id,
            text: event.text.clone(),
        });

        // Keep history limited (optional, e.g., last 100 messages)
        if chat_state.messages.len() > 100 {
            chat_state.messages.remove(0);
        }
    }
}

fn request_chat_history(
    mut client_history_writer: MessageWriter<ClientChatHistoryRequest>,
    mut chat_state: ResMut<ChatState>,
) {
    chat_state.history_loaded = false;
    client_history_writer.write(ClientChatHistoryRequest);
}

fn handle_chat_input(
    mut chat_state: ResMut<ChatState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut client_chat_writer: MessageWriter<ClientChat>,
) {
    // Open chat with Enter or T if not open
    if !chat_state.is_open {
        if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::KeyT) {
            chat_state.is_open = true;
            chat_state.has_focus = true;
        }
        return;
    }

    // Close with Escape
    if keyboard.just_pressed(KeyCode::Escape) {
        chat_state.is_open = false;
        chat_state.has_focus = false;
        return;
    }

    // Send with Enter
    if keyboard.just_pressed(KeyCode::Enter) && chat_state.has_focus {
        if !chat_state.input.trim().is_empty() {
            client_chat_writer.write(ClientChat {
                text: chat_state.input.clone(),
            });
            chat_state.input.clear();
        }
        // Keep focus or close after send? Usually keep focus in modern games,
        // but often close in simple MMOs. Let's keep it open for now,
        // or user can press Esc to close.
        // For now: clear input and keep focus.
    }
}

fn send_chat_identity(
    mut identity_writer: MessageWriter<ClientChatIdentity>,
    local_client_names: Query<&Name, With<LocalClient>>,
) {
    let name = local_client_names
        .iter()
        .next()
        .map(|name| name.as_str().to_string())
        .unwrap_or_else(|| "Player".to_string());

    identity_writer.write(ClientChatIdentity {
        name,
        steam_id: None,
    });
}

pub fn render_chat_ui(mut egui: EguiContexts, mut chat_state: ResMut<ChatState>) {
    // Always show chat history (maybe faded?) or only when open?
    // Let's emulate a typical MMO chat: Always visible background (transparent),
    // input only when "open".

    let Ok(ctx) = egui.ctx_mut() else {
        return;
    };

    // Define the window style
    let window_frame = egui::Frame::window(&ctx.style())
        .fill(egui::Color32::from_rgba_premultiplied(0, 50, 0, 150)) // Semi-transparent black
        .stroke(egui::Stroke::NONE)
        .inner_margin(5.0);

    egui::Window::new("Chat")
        .frame(window_frame)
        .anchor(egui::Align2::LEFT_BOTTOM, egui::Vec2::new(10.0, -10.0))
        .title_bar(false)
        .resizable(false)
        .fixed_size(egui::Vec2::new(300.0, 200.0))
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                // Chat History Area
                let scroll_area = egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .max_height(160.0); // Leave space for input

                scroll_area.show(ui, |ui| {
                    ui.set_min_width(ui.available_width());

                    for entry in &chat_state.messages {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!("{}:", entry.sender_name))
                                    .color(egui::Color32::LIGHT_BLUE)
                                    .strong(),
                            );
                            ui.label(egui::RichText::new(&entry.text).color(egui::Color32::WHITE));
                        });
                    }
                });

                // Input Area (only if open)
                if chat_state.is_open {
                    ui.separator();
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut chat_state.input)
                            .hint_text("Press Enter to send...")
                            .desired_width(f32::INFINITY)
                            .lock_focus(true), // Keep focus
                    );

                    if chat_state.has_focus {
                        response.request_focus();
                    }
                } else {
                    // Show a hint how to open chat if it's not empty?
                    // Or just invisible.
                    // ui.label(egui::RichText::new("Press [Enter] to chat").weak().small());
                }
            });
        });
}
