//! Utility methods for batch processing

use super::super::types::*;
use super::core::BatchProcessor;
use crate::utils::error::error::Result;
use chrono::Utc;

impl BatchProcessor {
    /// Update batch status
    pub(super) async fn update_batch_status(
        &self,
        batch_id: &str,
        status: BatchStatus,
    ) -> Result<()> {
        // Update in active batches
        {
            let mut active = self.active_batches.write().await;
            if let Some(batch) = active.get_mut(batch_id) {
                batch.status = status.clone();
            }
        }

        // Update in database
        self.database
            .update_batch_status(batch_id, &format!("{:?}", status))
            .await
    }

    /// Update batch progress
    pub(super) async fn update_batch_progress(
        &self,
        batch_id: &str,
        completed: u32,
        failed: u32,
    ) -> Result<()> {
        // Update in active batches
        {
            let mut active = self.active_batches.write().await;
            if let Some(batch) = active.get_mut(batch_id) {
                batch.request_counts.completed = completed as i32;
                batch.request_counts.failed = failed as i32;
            }
        }

        // Update in database
        self.database
            .update_batch_progress(batch_id, completed as i32, failed as i32)
            .await
    }

    /// Mark batch as completed
    pub(super) async fn mark_batch_completed(&self, batch_id: &str) -> Result<()> {
        let now = Utc::now();

        // Update in active batches
        {
            let mut active = self.active_batches.write().await;
            if let Some(batch) = active.get_mut(batch_id) {
                batch.completed_at = Some(now);
            }
        }

        // Update in database
        self.database.mark_batch_completed(batch_id).await
    }

    /// Check if batch is cancelled
    pub(super) async fn is_batch_cancelled(&self, batch_id: &str) -> Result<bool> {
        let active = self.active_batches.read().await;
        if let Some(batch) = active.get(batch_id) {
            Ok(matches!(
                batch.status,
                BatchStatus::Cancelling | BatchStatus::Cancelled
            ))
        } else {
            Ok(false)
        }
    }
}
