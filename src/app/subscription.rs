use super::*;
use std::time::Duration;

impl App {
    /// Registers global event listeners and maps them to [`Message`] values.
    pub fn subscription(&self) -> Subscription<Message> {
        let mut subs = vec![
            crate::subscriptions::keyboard::shortcuts(),
            crate::subscriptions::keyboard::modifier_state(),
            crate::subscriptions::keyboard::input_debug(),
            crate::subscriptions::mouse::sidebar_resize(),
            crate::subscriptions::window::resizes(),
            crate::subscriptions::window::focus_refresh(),
            iced::time::every(Duration::from_millis(150)).map(|_| Message::LspTick),
        ];

        if self.editor_preferences.autosave_enabled {
            subs.push(
                iced::time::every(Duration::from_millis(
                    self.editor_preferences.autosave_interval_ms.clamp(30, 1000),
                ))
                .map(|_| Message::AutosaveTick),
            );
        }

        if let Some(term) = &self.terminal_pane {
            subs.push(term.subscription().map(Message::TerminalEvent));
        }

        Subscription::batch(subs)
    }
}
