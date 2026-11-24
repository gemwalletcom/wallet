use std::cmp;
use std::time::Duration;

use storage::ParserState;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BlockPlanKind {
    Enqueue,
    Parse,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BlockRange {
    pub blocks: Vec<u64>,
    pub end_block: i64,
    pub remaining: i64,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BlockPlan {
    pub range: BlockRange,
    pub kind: BlockPlanKind,
}

pub fn timeout_for_state(state: &ParserState, min_check: Duration, max_check: Duration) -> Duration {
    let block_time = Duration::from_millis(state.block_time as u64);
    if block_time.is_zero() {
        return cmp::max(Duration::from_millis(state.timeout_latest_block as u64), min_check);
    }

    let remaining = block_time.saturating_sub(state.updated_at.elapsed());
    let upper = cmp::max(cmp::min(block_time, max_check), min_check);
    remaining.clamp(min_check, upper)
}

pub fn should_reload_catchup(processed_blocks: i64, interval: i64, elapsed: Duration, max_check: Duration) -> bool {
    (interval > 0 && processed_blocks >= interval) || elapsed >= max_check
}

pub fn plan_next_block(state: &ParserState, current_block: i64, latest_block: i64) -> Option<BlockPlan> {
    if !state.is_enabled {
        return None;
    }
    let start_block = current_block + 1;
    let end_block = cmp::min(start_block + state.parallel_blocks as i64 - 1, latest_block - state.await_blocks as i64);
    if end_block < start_block {
        return None;
    }
    let blocks = (start_block..=end_block).map(|b| b as u64).collect::<Vec<_>>();
    let remaining = latest_block - end_block - state.await_blocks as i64;
    let kind = if let Some(queue_behind_blocks) = state.queue_behind_blocks
        && remaining > queue_behind_blocks as i64
    {
        BlockPlanKind::Enqueue
    } else {
        BlockPlanKind::Parse
    };

    Some(BlockPlan {
        range: BlockRange { blocks, end_block, remaining },
        kind,
    })
}

#[cfg(test)]
mod tests {
    use super::{BlockPlan, BlockPlanKind, BlockRange, plan_next_block, should_reload_catchup, timeout_for_state};
    use primitives::DateTime;
    use std::time::Duration;
    use storage::ParserState;

    const MIN: Duration = Duration::from_secs(1);
    const MAX: Duration = Duration::from_secs(8);

    #[test]
    fn test_timeout_for_state_no_block_time() {
        let s = ParserState {
            timeout_latest_block: 500,
            ..ParserState::mock()
        };
        assert_eq!(timeout_for_state(&s, MIN, MAX), MIN);
        assert_eq!(timeout_for_state(&s, Duration::from_millis(100), MAX), Duration::from_millis(500));
    }

    #[test]
    fn test_timeout_for_state_uses_remaining_block_time() {
        let s = ParserState {
            block_time: 12_000,
            updated_at: DateTime::now() - Duration::from_secs(4),
            ..ParserState::mock()
        };
        let timeout = timeout_for_state(&s, MIN, MAX);
        assert!(timeout >= Duration::from_secs(7) && timeout <= Duration::from_secs(9));
    }

    #[test]
    fn test_timeout_for_state_caps_at_max() {
        let s = ParserState {
            block_time: 600_000,
            updated_at: DateTime::now(),
            ..ParserState::mock()
        };
        assert_eq!(timeout_for_state(&s, MIN, MAX), MAX);
    }

    #[test]
    fn test_timeout_for_state_overdue_block() {
        let s = ParserState {
            block_time: 10_000,
            updated_at: DateTime::now() - Duration::from_secs(15),
            ..ParserState::mock()
        };
        assert_eq!(timeout_for_state(&s, MIN, MAX), MIN);
    }

    #[test]
    fn test_should_reload_catchup_counts_processed_blocks() {
        for (processed, interval, expected) in [(10, 0, false), (4, 5, false), (5, 5, true), (11, 5, true), (49, 50, false), (50, 50, true), (56, 50, true)] {
            assert_eq!(should_reload_catchup(processed, interval, Duration::ZERO, MAX), expected);
        }
    }

    #[test]
    fn test_should_reload_catchup_limits_elapsed_time() {
        for interval in [0, 50] {
            for (elapsed, expected) in [(7, false), (8, true), (9, true)] {
                assert_eq!(should_reload_catchup(0, interval, Duration::from_secs(elapsed), MAX), expected);
            }
        }
    }

    #[test]
    fn test_catchup_batches_reload_without_aligned_remaining() {
        for (queue_behind_blocks, kind) in [(None, BlockPlanKind::Parse), (Some(0), BlockPlanKind::Enqueue)] {
            for (parallel_blocks, expected_batches, expected_processed) in [(5, 10, 50), (7, 8, 56)] {
                let mut state = ParserState {
                    current_block: 100,
                    latest_block: 1014,
                    await_blocks: 0,
                    parallel_blocks,
                    queue_behind_blocks,
                    ..ParserState::mock()
                };
                let mut processed = 0;
                let mut batches = 0;
                loop {
                    let plan = plan_next_block(&state, state.current_block, state.latest_block).unwrap();
                    assert_eq!(plan.kind, kind);
                    processed += plan.range.blocks.len() as i64;
                    state.current_block = plan.range.end_block;
                    batches += 1;
                    if should_reload_catchup(processed, 50, Duration::ZERO, MAX) {
                        break;
                    }
                }
                assert_eq!((batches, processed, state.current_block), (expected_batches, expected_processed, 100 + expected_processed));
            }
        }
    }

    #[test]
    fn test_plan_next_block_disabled_chain_resumes_from_checkpoint() {
        for (queue_behind_blocks, kind) in [(None, BlockPlanKind::Parse), (Some(0), BlockPlanKind::Enqueue)] {
            let mut state = ParserState {
                current_block: 100,
                latest_block: 200,
                await_blocks: 0,
                parallel_blocks: 5,
                queue_behind_blocks,
                ..ParserState::mock()
            };
            let completed = plan_next_block(&state, state.current_block, state.latest_block).unwrap();
            state.current_block = completed.range.end_block;
            state.is_enabled = false;
            assert_eq!(plan_next_block(&state, state.current_block, state.latest_block), None);
            state.is_enabled = true;
            assert_eq!(
                plan_next_block(&state, state.current_block, state.latest_block),
                Some(BlockPlan {
                    range: BlockRange {
                        blocks: vec![106, 107, 108, 109, 110],
                        end_block: 110,
                        remaining: 90
                    },
                    kind,
                })
            );
        }
    }

    #[test]
    fn test_plan_next_block_returns_none_when_no_blocks() {
        let state = ParserState {
            await_blocks: 5,
            parallel_blocks: 3,
            ..ParserState::mock()
        };
        let plan = plan_next_block(&state, 10, 12);
        assert!(plan.is_none());
    }

    #[test]
    fn test_plan_next_block_returns_none_when_disabled() {
        let state = ParserState {
            parallel_blocks: 3,
            is_enabled: false,
            ..ParserState::mock()
        };

        assert!(plan_next_block(&state, 5, 10).is_none());
    }

    #[test]
    fn test_plan_next_block_builds_expected_blocks() {
        let state = ParserState { parallel_blocks: 3, ..ParserState::mock() };
        let plan = plan_next_block(&state, 5, 10).unwrap();
        assert_eq!(plan.range.blocks, vec![6, 7, 8]);
        assert_eq!(plan.range.end_block, 8);
        assert_eq!(plan.range.remaining, 1);
        if let BlockPlanKind::Parse = plan.kind {
        } else {
            panic!("expected parse plan");
        }
    }

    #[test]
    fn test_plan_next_block_enqueues_when_behind() {
        let state = ParserState {
            parallel_blocks: 3,
            queue_behind_blocks: Some(2),
            ..ParserState::mock()
        };
        let plan = plan_next_block(&state, 5, 20).unwrap();
        if let BlockPlanKind::Enqueue = plan.kind {
        } else {
            panic!("expected enqueue plan");
        }
    }
}
