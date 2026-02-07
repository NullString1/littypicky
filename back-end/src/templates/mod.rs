pub fn get_email_verification_html() -> &'static str {
    include_str!("email_verification.html")
}

pub fn get_email_verification_text() -> &'static str {
    include_str!("email_verification.txt")
}

pub fn get_password_reset_html() -> &'static str {
    include_str!("password_reset.html")
}

pub fn get_password_reset_text() -> &'static str {
    include_str!("password_reset.txt")
}

pub fn get_password_reset_confirmation_html() -> &'static str {
    include_str!("password_reset_confirmation.html")
}

pub fn get_password_reset_confirmation_text() -> &'static str {
    include_str!("password_reset_confirmation.txt")
}

pub fn render_template(template: &str, replacements: &[(&str, &str)]) -> String {
    let mut result = template.to_string();
    for (key, value) in replacements {
        result = result.replace(key, value);
    }
    result
}
