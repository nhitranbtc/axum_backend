// Application use cases organized by domain
pub mod admin;
pub mod auth;
pub mod user;

// Re-export for backward compatibility
pub use auth::{
    ForgotPasswordUseCase, LoginUseCase, LogoutUseCase, RegisterUseCase, ResendConfirmCodeUseCase,
    SetPasswordUseCase, VerifyEmailUseCase,
};
pub use user::{
    CreateUserUseCase, GetUserRoleUseCase, GetUserUseCase, ImportUsersUseCase, ListUsersUseCase,
    UpdateUserRoleUseCase, UpdateUserUseCase,
};
