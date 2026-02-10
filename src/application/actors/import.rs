// Import the AuthRepository trait which provides database operations for user management
use crate::domain::repositories::AuthRepository;
// Import Ractor framework components:
// - Actor: The base trait that all actors must implement
// - ActorProcessingErr: Error type for actor processing failures
// - ActorRef: A reference to an actor that can be used to send messages
use ractor::{Actor, ActorProcessingErr, ActorRef};
use std::sync::Arc;

/// UserCreationActor is responsible for creating a single user in the database.
///
/// This actor follows the "one-shot" pattern - it processes exactly one user creation
/// message and then stops itself. This design ensures:
/// - Each user import gets its own isolated process
/// - No shared mutable state between user creations
/// - Clean resource cleanup after each operation
///
/// Generic Parameter:
/// - R: The repository type that implements AuthRepository and must live for 'static
///   (required by Ractor for actor safety across async boundaries)
pub struct UserCreationActor<R: AuthRepository + 'static> {
    /// Shared reference to the authentication repository
    /// Arc (Atomic Reference Counted) allows multiple actors to safely share
    /// the same repository instance without copying
    auth_repo: Arc<R>,
}

impl<R: AuthRepository + 'static> UserCreationActor<R> {
    /// Creates a new UserCreationActor instance
    ///
    /// # Arguments
    /// * `auth_repo` - Shared reference to the authentication repository
    ///
    /// # Returns
    /// A new actor instance ready to be spawned
    pub fn new(auth_repo: Arc<R>) -> Self {
        Self { auth_repo }
    }
}

/// UserCreationMsg represents the message sent to a UserCreationActor
///
/// In the Actor model, actors communicate exclusively through messages.
/// This struct contains all the data needed to create a user.
///
/// The Clone trait is required by Ractor for message passing
#[derive(Clone)]
pub struct UserCreationMsg {
    /// The user's email address (must be unique)
    pub email: String,
    /// The user's display name
    pub name: String,
    /// The pre-hashed password (hashing happens before sending the message)
    pub password_hash: String,
}

/// Implementation of the Actor trait for UserCreationActor
///
/// This is where we define how the actor behaves:
/// - What messages it can receive (Msg type)
/// - What state it maintains (State type)
/// - What arguments it needs to start (Arguments type)
/// - How it handles messages (handle method)
#[async_trait::async_trait]
impl<R: AuthRepository + 'static> Actor for UserCreationActor<R> {
    /// The type of messages this actor can receive
    type Msg = UserCreationMsg;

    /// The actor's internal state
    /// We use () because this actor is stateless - it doesn't need to remember
    /// anything between messages (and it only processes one message anyway)
    type State = ();

    /// Arguments passed when spawning the actor
    /// We use () because all necessary data is in the actor struct itself
    type Arguments = ();

    /// Called once when the actor starts, before it can receive messages
    ///
    /// This is the actor's initialization phase. Here we could:
    /// - Set up resources
    /// - Initialize state
    /// - Perform validation
    ///
    /// # Arguments
    /// * `_myself` - Reference to this actor (unused, hence the _ prefix)
    /// * `_args` - Arguments passed during spawn (unused)
    ///
    /// # Returns
    /// Ok(state) if initialization succeeds, Err if it fails
    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        // No initialization needed, return empty state
        Ok(())
    }

    /// Handles incoming messages - this is the core business logic
    ///
    /// This method is called each time the actor receives a message.
    /// For our one-shot actor, it will only be called once.
    ///
    /// # Arguments
    /// * `_myself` - Reference to this actor (used to stop it after processing)
    /// * `msg` - The UserCreationMsg containing user data
    /// * `_state` - Mutable reference to the actor's state (unused)
    ///
    /// # Returns
    /// Ok(()) if message processing succeeds, Err if it fails
    ///
    /// # Process Flow
    /// 1. Check if user already exists (duplicate prevention)
    /// 2. If user doesn't exist, create them in the database
    /// 3. Log the result
    /// 4. Stop the actor (one-shot pattern)
    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        msg: Self::Msg,
        _state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        // Step 1: Check if a user with this email already exists
        // This prevents duplicate user creation and maintains data integrity
        let exists = self
            .auth_repo
            .find_by_email(&msg.email)
            .await
            .map_err(|e| ActorProcessingErr::from(e.to_string()))?;

        // Step 2: Only create the user if they don't already exist
        if exists.is_none() {
            // Call the repository to create the user in the database
            // The password is already hashed, so we pass it directly
            self.auth_repo
                .create_user(
                    &msg.email,
                    &msg.name,
                    Some(msg.password_hash.clone()), // Pass existing option
                    None,                            // confirmation_code
                    None,                            // expires_at
                )
                .await
                .map_err(|e| ActorProcessingErr::from(e.to_string()))?;

            // Log success for monitoring and debugging
            tracing::info!("Actor: Successfully created user: {}", msg.email);
        } else {
            // User already exists, skip creation and log it
            tracing::info!("Actor: User already exists: {}", msg.email);
        }

        // Step 3: Stop this actor after processing one message (one-shot pattern)
        // This ensures clean resource cleanup and prevents the actor from
        // processing additional messages
        //
        // The None parameter means "stop gracefully without a specific reason"
        _myself.stop(None);

        Ok(())
    }
}
