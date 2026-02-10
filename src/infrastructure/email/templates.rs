use askama::Template;

#[derive(Template)]
#[template(path = "welcome.html")]
pub struct WelcomeTemplate {
    pub name: String,
}

#[derive(Template)]
#[template(path = "confirmation.html")]
pub struct ConfirmationTemplate {
    pub name: String,
    pub code: String,
}

#[derive(Template)]
#[template(path = "forgot_password.html")]
pub struct ForgotPasswordTemplate {
    pub name: String,
    pub code: String,
}
