// Copyright (c). Gem Wallet. All rights reserved.

import os
import Testing
@testable import WalletConnectorService

struct SetupStateTests {
    @Test
    func startsOnce() async {
        let starts = OSAllocatedUnfairLock(initialState: 0)
        let state = SetupState()

        await state.start {
            starts.withLock { $0 += 1 }
            return Task {}
        }
        await state.start {
            starts.withLock { $0 += 1 }
            return Task {}
        }

        #expect(starts.withLock { $0 } == 1)
    }
}
