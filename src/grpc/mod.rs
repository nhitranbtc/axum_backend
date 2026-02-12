pub mod application;
pub mod config;
pub mod errors;
pub mod infrastructure;
pub mod presentation;

// Keeping old modules temporarily for backward compatibility while migrating
// pub mod user_service;
// pub mod interceptors;
// pub mod actors;

// Include the generated protobuf code
pub mod proto {
    tonic::include_proto!("user");
}
