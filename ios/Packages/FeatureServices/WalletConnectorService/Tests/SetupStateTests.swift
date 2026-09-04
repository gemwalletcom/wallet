// Copyright (c). Gem Wallet. All rights reserved.

import os
import Testing
@testable import WalletConnectorService

struct SetupStateTests {
    @Test
    func runsOnceBeforeConcurrentCallersReturn() async {
        let callerCount = 20
        let observations = OSAllocatedUnfairLock(initialState: (
            starts: 0,
            isReady: false,
            readyCallers: 0,
        ))
        let state = SetupState()

        await withTaskGroup(of: Void.self) { group in
            for _ in 0..<callerCount {
                group.addTask {
                    await state.runOnce {
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
}
