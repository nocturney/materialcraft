pub(super) enum LibraryDividerAction {
    /// Dragged to this tree width
    Dragged(f32),
    /// Double-clicked: back to the automatic width
    Reset,
}
