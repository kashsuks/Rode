use crate::features::search::SearchResult;
use iced_code_editor::LspOverlayMessage;
use iced_term::Event as TerminalEvent;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    /// Text editing stuff — forwarded from iced-code-editor
    CodeEditorEvent(iced_code_editor::Message),
    /// LSP overlay events (hover, completion selection)
    LspOverlay(LspOverlayMessage),
    /// Content changed notification (text, is_modified) for bookkeeping
    CodeEditorContentChanged,
    FileClicked(PathBuf),
    FileOpened(PathBuf, String),
    SensitiveFileOpenConfirm(bool),
    FolderToggled(PathBuf),
    FileTreeRefresh,
    ToggleSidebar,
    OpenFileDialog,
    OpenFolderDialog,
    FolderOpened(PathBuf),
    SaveFile,
    SaveCurrentFileAs(PathBuf),
    CurrentFileSavedAs(PathBuf),
    FileSaved(Result<(), String>),
    InputLog(String),

    TabSelected(usize),
    TabClosed(usize),
    CloseActiveTab,

    SidebarResizeStart,
    SidebarResizing(f32),
    SidebarResizeEnd,

    PreviewMarkdown,
    MarkdownLinkClicked(iced::widget::markdown::Uri),

    ToggleSearch,
    SearchQueryChanged(String),
    SearchCompleted(Vec<SearchResult>),
    SearchResultClicked(PathBuf, usize),
    /// File finding (Cmd+T, legacy)
    ToggleFileFinder,
    FileFinderQueryChanged(String),
    FileFinderSelect,
    FileFinderNavigate(i32),
    /// Fuzzy Finder (Cmd+Shift+F)
    ToggleFuzzyFinder,
    FuzzyFinderQueryChanged(String),
    FuzzyFinderSelect,
    FuzzyFinderNavigate(i32),
    /// Fullscreen and window management stuff
    ToggleFullscreen(iced::window::Mode),
    EscapePressed,
    /// Command Palette (Cmd+Shift+P)
    ToggleCommandPalette,
    CommandPaletteQueryChanged(String),
    CommandPaletteSelect(String),
    CommandPaletteNavigate(i32),
    /// Embedded terminal events
    TerminalEvent(TerminalEvent),
    /// Terminal panel (Cmd/Ctrl+J)
    ToggleTerminal,
    /// Explicit focus switching between editor and terminal panels
    FocusEditor,
    FocusTerminal,
    /// Find and Replace (Cmd+F)
    ToggleFindReplace,
    FindQueryChanged(String),
    ReplaceQueryChanged(String),
    FindNext,
    FindPrev,
    ReplaceOne,
    ReplaceAll,
    ToggleCaseSensitive,
    /// Settings panel
    ToggleSettings,
    SettingsNavigate(String),
    SettingsTabSizeChanged(String),
    SettingsToggleUseSpaces,
    SettingsToggleAutosave,
    SettingsAutosaveIntervalChanged(String),
    SettingsSavePreferences,
    SettingsSelectTheme(String),
    SettingsReloadTheme,
    SettingsLineNumberWidthChanged(String),
    /// Vim-style command input
    ToggleCommandInput,
    CommandInputChanged(String),
    CommandInputSubmit,
    /// Window resize event
    WindowResized(u32, u32),
    /// New file
    NewFile,
    SaveAs,
    /// WakaTime
    WakaTimeApiKeyChanged(String),
    WakaTimeApiKeyHoverStart,
    WakaTimeApiKeyHoverEnd,
    WakaTimeApiUrlChanged(String),
    SaveWakaTimeSettings,

    DismissNotification,
    ModifierStateChanged(iced::keyboard::Modifiers),
    LspTick,
    AutosaveTick,
    AutosaveFinished(PathBuf, String, Result<(), String>),

    // Developer mode
    ToggleDeveloperPanel,
    ClearDeveloperLogs,
    SettingsToggleDeveloperMode,
    ToggleLsp,

    // Updater
    CheckForUpdate,
    UpdateAvailable(crate::features::updater::UpdateInfo),
    DismissUpdateBanner,
}
