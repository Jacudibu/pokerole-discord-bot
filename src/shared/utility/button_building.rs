use serenity::all::{ButtonStyle, CreateButton};

pub fn create_styled_button(
    label: &str,
    custom_id: &str,
    is_disabled: bool,
    style: ButtonStyle,
) -> CreateButton {
    create_button(label, custom_id, is_disabled).style(style)
}

pub fn create_button(label: &str, custom_id: &str, is_disabled: bool) -> CreateButton {
    CreateButton::new(custom_id)
        .label(label)
        .style(ButtonStyle::Primary)
        .disabled(is_disabled)
}
