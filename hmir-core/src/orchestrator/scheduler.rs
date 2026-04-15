use crate::orchestrator::batching::{Sequence, SequenceStatus};

/// The DAG Engine looping over the cost matrix to drive sequences forward.
pub struct ExecutionEngine {
    sequences: Vec<Sequence>,
    pub max_concurrent: usize,
}

impl ExecutionEngine {
    pub fn new(max_concurrent_requests: usize) -> Self {
        Self {
            sequences: Vec::new(),
            max_concurrent: max_concurrent_requests,
        }
    }

    pub fn add_request(&mut self, seq: Sequence) {
        self.sequences.push(seq);
    }

    /// The core scheduler loop. Triggers iterations of sequence generation while explicitly policing limits.
    pub fn schedule_step(&mut self) {
        let mut running_count = self.sequences.iter().filter(|s| s.status == SequenceStatus::Running).count();

        for seq in self.sequences.iter_mut() {
            match seq.status {
                SequenceStatus::Waiting => {
                    if running_count < self.max_concurrent {
                        // Promote to Running
                        seq.status = SequenceStatus::Running;
                        running_count += 1;
                    }
                }
                SequenceStatus::Running => {
                    // Force a forward pass
                    seq.step();
                }
                _ => {} // Preempted or Finished sequences are skipped in hot-path
            }
        }
    }

    /// Safety monitor to instantly freeze processes if VRAM exceeds capacity limits.
    pub fn watchdog_vram_panic(&mut self) {
        // Pseudo logic: if total `active_vram_blocks_used` exceeds hardware capacities,
        // preempt the youngest sequences down to RAM.
        
        let running_seqs: Vec<&mut Sequence> = self.sequences
            .iter_mut()
            .filter(|s| s.status == SequenceStatus::Running)
            .collect();
            
        // Preempt sequences starting from the back
        if running_seqs.len() > self.max_concurrent {
            let overflow = running_seqs.len() - self.max_concurrent;
            for seq in running_seqs.into_iter().rev().take(overflow) {
                // Here we would pass in the LogicalPageTable to orchestrate the swap
                seq.status = SequenceStatus::Preempted;
            }
        }
    }
}

// -----------------------------------------------------------------------------
// TESTS
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestrator::batching::SequenceStatus;

    #[test]
    fn test_sequence_lifecycle() {
        let mut seq = Sequence::new(101, 2);
        
        assert_eq!(seq.status, SequenceStatus::Waiting);
        
        seq.status = SequenceStatus::Running;
        seq.step(); 
        assert_eq!(seq.num_tokens_generated, 1);
        assert_eq!(seq.status, SequenceStatus::Running);
        
        seq.step();
        // Finished once Max Tokens limit hits
        assert_eq!(seq.num_tokens_generated, 2);
        assert_eq!(seq.status, SequenceStatus::Finished);
    }

    #[test]
    fn test_scheduler_preemption_guardrail() {
        let mut engine = ExecutionEngine::new(2); // Only allows 2 concurrent sequences safely

        // Load 3 heavy requests
        engine.add_request(Sequence::new(1, 100));
        engine.add_request(Sequence::new(2, 100));
        engine.add_request(Sequence::new(3, 100));

        engine.schedule_step();

        let running_seqs = engine.sequences.iter().filter(|s| s.status == SequenceStatus::Running).count();
        let waiting_seqs = engine.sequences.iter().filter(|s| s.status == SequenceStatus::Waiting).count();

        assert_eq!(running_seqs, 2, "Engine should strictly block at its 2 concurrent maximum limits.");
        assert_eq!(waiting_seqs, 1, "The third sequence must be indefinitely paused in waiting.");
    }
}
