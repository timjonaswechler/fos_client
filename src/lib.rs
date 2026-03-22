pub mod chat;
pub mod debug;

// =============================================================================
// Steam Configuration - Wird von build.rs generiert
// =============================================================================
// Dieses Modul enthält die Steam App ID und Build-Konfiguration.
// Die Werte werden zur Compile-Zeit aus der Umgebungsvariable STEAM_APP_ID
// oder der Datei steam_appid.txt gelesen.
//
// In CI/CD wird STEAM_APP_ID aus GitHub Secrets gesetzt.
// Lokal kann die Datei steam_appid.txt verwendet werden (nicht committen!).
// =============================================================================
pub mod config {
    include!(concat!(env!("OUT_DIR"), "/config.rs"));
}

// Re-exports für einfachen Zugriff
pub use config::BUILD_PROFILE;
pub use config::IS_RELEASE;
pub use config::STEAM_APP_ID;

use {
    bevy::prelude::*,
    chat::ChatPlugin,
    chicken::ChickenPlugin,
    chicken::identity::PlayerIdentity,
    chicken::network::client::LocalIdentity,
    chicken::notifications::{NotificationQueue, notification_lifecycle, on_notify},
    serde::{Deserialize, Serialize},
};

pub struct FOSClientPlugin;

#[derive(Event, Serialize, Message, Deserialize)]
pub struct DummyEvent;

impl Plugin for FOSClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ChickenPlugin,
            // ProtocolPlugin,
            ChatPlugin,
            // ChickenNotificationPlugin,
        ))
        .init_resource::<NotificationQueue>()
        .add_observer(on_notify)
        .add_systems(Update, notification_lifecycle)
        .add_systems(PostStartup, init_player_identity);
    }
}

/// Initialisiert `PlayerIdentity` aus der lokalen `LocalIdentity` (Ed25519-Key).
/// Solange kein echter Spielername (Steam, Profil) gesetzt ist, wird Player-{id} verwendet.
fn init_player_identity(
    mut commands: Commands,
    local_identity: Option<Res<LocalIdentity>>,
) {
    let Some(local_id) = local_identity else {
        return;
    };
    commands.insert_resource(PlayerIdentity::local(
        format!("Player-{}", &local_id.player_id[..8]),
    ));
}
