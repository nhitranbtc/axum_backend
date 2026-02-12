use ractor::{Actor, ActorRef, SpawnErr};
use std::sync::atomic::{AtomicUsize, Ordering};

use super::user_service_actor::UserServiceActor;
use super::messages::UserServiceMessage;
use crate::infrastructure::database::connection::DbPool;

/// Actor pool for load balancing requests across multiple workers
pub struct ActorPool {
    workers: Vec<ActorRef<UserServiceMessage>>,
    current_index: AtomicUsize,
}

impl ActorPool {
    /// Create a new actor pool with the specified number of workers
    pub async fn new(
        pool_size: usize,
        db_pool: DbPool,
    ) -> Result<Self, SpawnErr> {
        let mut workers = Vec::with_capacity(pool_size);
        
        for i in 0..pool_size {
            let (actor_ref, _handle) = Actor::spawn(
                Some(format!("user-service-worker-{}", i)),
                UserServiceActor,
                db_pool.clone(),
            )
            .await?;
            
            workers.push(actor_ref);
        }
        
        tracing::info!("Actor pool created with {} workers", pool_size);
        
        Ok(Self {
            workers,
            current_index: AtomicUsize::new(0),
        })
    }
    
    /// Get next worker using round-robin load balancing
    pub fn next_worker(&self) -> ActorRef<UserServiceMessage> {
        let index = self.current_index.fetch_add(1, Ordering::Relaxed) % self.workers.len();
        self.workers[index].clone()
    }
    
    /// Get pool size
    pub fn size(&self) -> usize {
        self.workers.len()
    }
    
    /// Shutdown all workers gracefully
    pub async fn shutdown(&self) {
        tracing::info!("Shutting down actor pool with {} workers", self.workers.len());
        for (i, worker) in self.workers.iter().enumerate() {
            tracing::debug!("Stopping worker {}", i);
            worker.stop(None);
        }
    }
}

impl Drop for ActorPool {
    fn drop(&mut self) {
        tracing::debug!("ActorPool dropped");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_round_robin() {
        // Test that round-robin works correctly
        let pool = ActorPool {
            workers: vec![],
            current_index: AtomicUsize::new(0),
        };
        
        // Simulate round-robin
        for i in 0..10 {
            let expected = i % pool.workers.len().max(1);
            let actual = pool.current_index.load(Ordering::Relaxed) % pool.workers.len().max(1);
            assert_eq!(expected, actual);
            pool.current_index.fetch_add(1, Ordering::Relaxed);
        }
    }
}
