//! The shape of a phone, an e-mail and a zip code, shared by the modules (ADR-029).

/// Ten digits (a Mexican number), or up to thirteen with the country code or an extension.
pub fn phone_ok(s: &str) -> bool {
    (10..=13).contains(&s.chars().filter(char::is_ascii_digit).count())
}

pub fn email_ok(s: &str) -> bool {
    let s = s.trim();
    match s.split_once('@') {
        Some((u, d)) => !u.is_empty() && !d.contains('@') && d.contains('.') && !d.starts_with('.') && !d.ends_with('.') && !s.contains(char::is_whitespace),
        None => false,
    }
}

pub fn zip_ok(s: &str) -> bool {
    let s = s.trim();
    s.len() == 5 && s.bytes().all(|b| b.is_ascii_digit())
}
