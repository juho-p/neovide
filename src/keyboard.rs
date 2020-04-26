use skulpin::winit::event::{ModifiersState, VirtualKeyCode};

pub fn transform_character(c: char, modifiers: &ModifiersState) -> Option<String> {
    let modifier = if modifiers.alt() { "M" }
        // don't handle ctrl here
        else { "" };

    match c {
        '\u{7f}' => None, // Del
        '<' => Some(if modifier.is_empty() {
            "<lt>".to_string()
        } else {
            format!("<{}-lt>", modifier)
        }),
        _ => Some(if modifier.is_empty() {
            c.to_string()
        } else {
            format!("<{}-{}>", modifier, c)
        }),
    }
}

pub fn transform_keycode(code: VirtualKeyCode, modifiers: &ModifiersState) -> Option<String> {
    let modifier = if modifiers.alt() {
        "M"
    } else if modifiers.ctrl() {
        "C"
    } else {
        ""
    };

    let key_str = match code {
        VirtualKeyCode::F1 => Some("F1"),
        VirtualKeyCode::F2 => Some("F2"),
        VirtualKeyCode::F3 => Some("F3"),
        VirtualKeyCode::F4 => Some("F4"),
        VirtualKeyCode::F5 => Some("F5"),
        VirtualKeyCode::F6 => Some("F6"),
        VirtualKeyCode::F7 => Some("F7"),
        VirtualKeyCode::F8 => Some("F8"),
        VirtualKeyCode::F9 => Some("F9"),
        VirtualKeyCode::F10 => Some("F10"),
        VirtualKeyCode::F11 => Some("F11"),
        VirtualKeyCode::F12 => Some("F12"),
        VirtualKeyCode::Insert => Some("Insert"),
        VirtualKeyCode::Home => Some("Home"),
        VirtualKeyCode::Delete => Some("Delete"),
        VirtualKeyCode::End => Some("End"),
        VirtualKeyCode::PageDown => Some("PageDown"),
        VirtualKeyCode::PageUp => Some("PageUp"),
        VirtualKeyCode::Up => Some("Up"),
        VirtualKeyCode::Down => Some("Down"),
        VirtualKeyCode::Left => Some("Left"),
        VirtualKeyCode::Right => Some("Right"),
        _ => None,
    };

    key_str.map(|s| {
        if modifier.is_empty() {
            format!("<{}>", s)
        } else {
            format!("<{}-{}>", modifier, s)
        }
    })
}
