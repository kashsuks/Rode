use super::*;

impl App {
    /// Registers global event listeners and maps them to [`Message`] values.
    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            crate::subscriptions::keyboard::shortcuts(),
            crate::subscriptions::mouse::sidebar_resize(),
        ])
    }
}
