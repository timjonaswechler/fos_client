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
pub mod steam_config {
    include!(concat!(env!("OUT_DIR"), "/steam_config.rs"));
}

// Re-exports für einfachen Zugriff
pub use steam_config::BUILD_PROFILE;
pub use steam_config::IS_RELEASE;
pub use steam_config::STEAM_APP_ID;

use {
    bevy::prelude::*,
    chat::ChatPlugin,
    chicken::network::ChickenNetPlugin,
    chicken::notifications::{
        ChickenNotificationPlugin, NotificationQueue, notification_lifecycle, on_notify,
    },
    chicken::protocols::ProtocolPlugin,
    chicken::states::StatusManagementPlugin,
    serde::{Deserialize, Serialize},
};

pub struct FOSClientPlugin;

#[derive(Event, Serialize, Message, Deserialize)]
pub struct DummyEvent;

impl Plugin for FOSClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            StatusManagementPlugin,
            ChickenNetPlugin,
            ProtocolPlugin,
            ChatPlugin,
            ChickenNotificationPlugin,
        ))
        .init_resource::<NotificationQueue>()
        .add_observer(on_notify)
        .add_systems(Update, notification_lifecycle);
    }
}
