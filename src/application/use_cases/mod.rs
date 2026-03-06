// Application use cases organized by domain
pub mod admin;
pub mod auth;
pub mod post;
pub mod user;

// Re-export for backward compatibility
pub use auth::{
    ForgotPasswordUseCase, LoginUseCase, LogoutUseCase, RegisterUseCase, ResendConfirmCodeUseCase,
    SetPasswordUseCase, VerifyEmailUseCase,
};
pub use post::{
    CreatePostUseCase, DeletePostUseCase, GetPostUseCase, ListPostsUseCase, UpdatePostUseCase,
};
pub use user::{
    CreateUserUseCase, DeleteUserUseCase, GetUserRoleUseCase, GetUserUseCase, ImportUsersUseCase,
    ListUsersUseCase, UpdateUserRoleUseCase, UpdateUserUseCase,
};
