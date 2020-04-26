use skulpin::winit::event::{VirtualKeyCode};

pub fn transform_character(c: char) -> Option<String> {
    match c {
        '<' => Some("<lt>".to_string()),
        '\u{7f}' => None, // Del
        _ => Some(c.to_string()),
    }
}

pub fn transform_keycode(code: VirtualKeyCode) -> Option<String> {
    let s = match code {
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

    s.map(|s| format!("<{}>", s))
}
