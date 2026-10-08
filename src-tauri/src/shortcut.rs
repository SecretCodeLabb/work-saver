//! Atajos de teclado en texto ("Ctrl+Shift+S") ↔ códigos de tecla virtual de Windows.

const MODIFIERS: [(&str, u16); 4] = [("Ctrl", 0x11), ("Shift", 0x10), ("Alt", 0x12), ("Win", 0x5B)];

/// Convierte un atajo en la lista de teclas virtuales a pulsar (modificadores primero).
pub fn parse(text: &str) -> Option<Vec<u16>> {
    let mut modifiers = [false; 4];
    let mut key: Option<u16> = None;

    for token in text.split('+').map(str::trim).filter(|t| !t.is_empty()) {
        if let Some(index) = modifier_index(token) {
            modifiers[index] = true;
        } else if key.is_none() {
            key = Some(key_code(token)?);
        } else {
            return None; // Solo se admite una tecla principal.
        }
    }

    let mut keys: Vec<u16> = MODIFIERS
        .iter()
        .zip(modifiers)
        .filter(|(_, on)| *on)
        .map(|((_, vk), _)| *vk)
        .collect();
    keys.push(key?);
    Some(keys)
}

/// Devuelve el atajo con formato canónico ("ctrl + s" → "Ctrl+S") o `None` si no es válido.
pub fn normalize(text: &str) -> Option<String> {
    let keys = parse(text)?;
    let (main, modifiers) = keys.split_last()?;
    let mut parts: Vec<String> = MODIFIERS
        .iter()
        .filter(|(_, vk)| modifiers.contains(vk))
        .map(|(name, _)| name.to_string())
        .collect();
    parts.push(key_name(*main));
    Some(parts.join("+"))
}

fn modifier_index(token: &str) -> Option<usize> {
    match token.to_ascii_lowercase().as_str() {
        "ctrl" | "control" => Some(0),
        "shift" => Some(1),
        "alt" => Some(2),
        "win" | "meta" | "super" => Some(3),
        _ => None,
    }
}

const NAMED_KEYS: [(&str, u16); 10] = [
    ("Enter", 0x0D),
    ("Space", 0x20),
    ("Tab", 0x09),
    ("Esc", 0x1B),
    ("Insert", 0x2D),
    ("Delete", 0x2E),
    ("Home", 0x24),
    ("End", 0x23),
    ("PageUp", 0x21),
    ("PageDown", 0x22),
];

fn key_code(token: &str) -> Option<u16> {
    let upper = token.to_ascii_uppercase();
    if upper.len() == 1 {
        let c = upper.as_bytes()[0];
        if c.is_ascii_uppercase() || c.is_ascii_digit() {
            return Some(u16::from(c));
        }
    }
    if let Some(n) = upper.strip_prefix('F').and_then(|n| n.parse::<u16>().ok()) {
        if (1..=24).contains(&n) {
            return Some(0x70 + n - 1);
        }
    }
    NAMED_KEYS
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(token))
        .map(|(_, vk)| *vk)
}

fn key_name(vk: u16) -> String {
    match vk {
        0x30..=0x39 | 0x41..=0x5A => char::from(vk as u8).to_string(),
        0x70..=0x87 => format!("F{}", vk - 0x70 + 1),
        _ => NAMED_KEYS
            .iter()
            .find(|(_, code)| *code == vk)
            .map(|(name, _)| name.to_string())
            .unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_shortcuts() {
        assert_eq!(parse("Ctrl+S"), Some(vec![0x11, 0x53]));
        assert_eq!(parse("ctrl + alt + s"), Some(vec![0x11, 0x12, 0x53]));
        assert_eq!(parse("F2"), Some(vec![0x71]));
        assert_eq!(parse("Ctrl"), None);
        assert_eq!(parse("Ctrl+S+D"), None);
        assert_eq!(parse("Ctrl+Ñ"), None);
    }

    #[test]
    fn normalizes_order_and_case() {
        assert_eq!(normalize("shift + ctrl + s").as_deref(), Some("Ctrl+Shift+S"));
        assert_eq!(normalize("alt+f12").as_deref(), Some("Alt+F12"));
        assert_eq!(normalize("ctrl+pagedown").as_deref(), Some("Ctrl+PageDown"));
    }
}
