#[must_use]
pub fn get_email_verification_html() -> &'static str {
    include_str!("email_verification.html")
}

#[must_use]
pub fn get_email_verification_text() -> &'static str {
    include_str!("email_verification.txt")
}

#[must_use]
pub fn get_password_reset_html() -> &'static str {
    include_str!("password_reset.html")
}

#[must_use]
pub fn get_password_reset_text() -> &'static str {
    include_str!("password_reset.txt")
}

#[must_use]
pub fn get_password_reset_confirmation_html() -> &'static str {
    include_str!("password_reset_confirmation.html")
}

#[must_use]
pub fn get_password_reset_confirmation_text() -> &'static str {
    include_str!("password_reset_confirmation.txt")
}

#[must_use]
pub fn render_template(template: &str, replacements: &[(&str, &str)]) -> String {
    let mut result = template.to_string();
    for (key, value) in replacements {
        result = result.replace(key, value);
    }
    result
}
