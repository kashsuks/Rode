/// This file is responsible for internal messages
/// Used to send internal flags and data transfer to trigger
/// Other instances of Message types.

use crate::features::search::SearchResult;
use iced_code_editor::LspOverlayMessage;
use iced_term::Event as TerminalEvent;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    CodeEditorEvent(iced_code_editor::Message),

    LspOverlay(LspOverlayMessage),

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

    ToggleFileFinder,
    FileFinderQueryChanged(String),
    FileFinderSelect,
    FileFinderNavigate(i32),

    ToggleFuzzyFinder,
    FuzzyFinderQueryChanged(String),
    FuzzyFinderSelect,
    FuzzyFinderNavigate(i32),

    ToggleFullscreen(iced::window::Mode),
    EscapePressed,

    ToggleCommandPalette,
    CommandPaletteQueryChanged(String),
    CommandPaletteSelect(String),
    CommandPaletteNavigate(i32),

    TerminalEvent(TerminalEvent),
    ToggleTerminal,
    FocusEditor,
    FocusTerminal,

    ToggleFindReplace,
    FindQueryChanged(String),
    ReplaceQueryChanged(String),
    FindNext,
    FindPrev,
    ReplaceOne,
    ReplaceAll,
    ToggleCaseSensitive,

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

    ToggleCommandInput,
    CommandInputChanged(String),
    CommandInputSubmit,
    WindowResized(u32, u32),

    NewFile,
    SaveAs,

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

    ToggleDeveloperPanel,
    ClearDeveloperLogs,
    SettingsToggleDeveloperMode,
    ToggleLsp,

    CheckForUpdate,
    UpdateAvailable(crate::features::updater::UpdateInfo),
    DismissUpdateBanner,
}
