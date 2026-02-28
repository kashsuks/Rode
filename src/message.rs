use iced::widget::text_editor::Action;
use std::path::PathBuf;
use crate::search::SearchResult;

#[derive(Debug, Clone)]
pub enum Message {
    /// Text editing stuff
    EditorAction(Action),
    FileClicked(PathBuf),
    FileOpened(PathBuf, String),
    FolderToggled(PathBuf),
    FileTreeRefresh,
    ToggleSidebar,
    OpenFolderDialog,
    FolderOpened(PathBuf),
    SaveFile,
    FileSaved(Result<(), String>),

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
    /// Terminal launcher (Cmd+J)
    ToggleTerminal,
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
    SettingsSavePreferences,
    SettingsSelectTheme(String),
    SettingsReloadTheme,
    /// Vim-style command input
    ToggleCommandInput,
    CommandInputChanged(String),
    CommandInputSubmit,
    /// New file
    NewFile,
    SaveAs,
    /// WakaTime
    WakaTimeApiKeyChanged(String),
    WakaTimeApiUrlChanged(String),
    SaveWakaTimeSettings,

    DismissNotification,

    // Updater
    CheckForUpdate,
    UpdateAvailable(crate::updater::UpdateInfo),
    DismissUpdateBanner,
}