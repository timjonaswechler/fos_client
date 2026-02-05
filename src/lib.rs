pub mod chat;
pub mod debug;

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
