use crate::memory::allocator::LogicalPageTable;

/// Tracks exact status of an LLM request during continuous batching.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SequenceStatus {
    /// Newly arrived, awaiting prompt/pre-fill computation
    Waiting,
    /// Actively processing tokens directly in VRAM
    Running,
    /// Paused and swapped to RAM due to Memory/VRAM limits
    Preempted,
    /// Completed generation constraint or EOS token hit
    Finished,
}

/// A Sequence represents an individual chat/inference request undergoing generation.
#[derive(Debug)]
pub struct Sequence {
    pub id: u64,
    pub status: SequenceStatus,
    pub num_tokens_generated: usize,
    pub max_tokens: usize,
    pub active_vram_blocks_used: usize,
}

impl Sequence {
    pub fn new(id: u64, max_tokens: usize) -> Self {
        Self {
            id,
            status: SequenceStatus::Waiting,
            num_tokens_generated: 0,
            max_tokens,
            active_vram_blocks_used: 0,
        }
    }

    /// Progresses a Sequence forward one token step.
    pub fn step(&mut self) -> bool {
        if self.status != SequenceStatus::Running {
            return false;
        }
        
        self.num_tokens_generated += 1;
        if self.num_tokens_generated >= self.max_tokens {
            self.status = SequenceStatus::Finished;
        }
        
        true
    }

    /// Suspend sequence and signal memory manager that its VRAM block is cold.
    pub fn preempt(&mut self, _pager_tracker: &mut LogicalPageTable) {
        self.status = SequenceStatus::Preempted;
        // In a full implementation, we locate the Sequence's exact BlockIds mapped
        // inside the Page Table and explicitly command them to swap to system RAM.
        // `pager_tracker.evict_coldest_to_ram()` acts as a fallback if this explicit swap fails.
    }

    /// Wake a sequence up. Requires restoring KV cache layers back into VRAM.
    pub fn resume(&mut self, _pager_tracker: &mut LogicalPageTable) {
        self.status = SequenceStatus::Waiting; // Moves back into execution queue
    }
}
