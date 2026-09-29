// Copyright (c). Gem Wallet. All rights reserved.

import os
import Testing
@testable import WalletConnectorService

struct SetupStateTests {
    @Test
    func startsOnceBeforeConcurrentCallersReturn() async {
        let callerCount = 20
        let observations = OSAllocatedUnfairLock(initialState: (
            starts: 0,
            isReady: false,
            readyCallers: 0,
        ))
        let state = SetupState()

        await withTaskGroup(of: Void.self) { group in
            for _ in 0 ..< callerCount {
                group.addTask {
                    await state.start {
                        observations.withLock {
                            $0.starts += 1
                            $0.isReady = true
                        }
                    }
                    observations.withLock {
                        if $0.isReady {
                            $0.readyCallers += 1
                        }
                    }
                }
            }
        }

        let result = observations.withLock { $0 }
        #expect(result.starts == 1)
        #expect(result.readyCallers == callerCount)
    }

    @Test
    func failedStartRunsAgainOnNextCall() async throws {
        let state = SetupState()
        let starts = OSAllocatedUnfairLock(initialState: 0)

        await #expect(throws: CancellationError.self) {
            try await state.start {
                starts.withLock { $0 += 1 }
                throw CancellationError()
            }
        }
        try await state.start {
            starts.withLock { $0 += 1 }
        }
        try await state.start {
            starts.withLock { $0 += 1 }
        }

        #expect(starts.withLock { $0 } == 2)
    }
}
