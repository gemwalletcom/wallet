// Copyright (c). Gem Wallet. All rights reserved.

import ConnectionStatusService
import class Gemstone.GemConnectionService

public extension ConnectionStatusObserver {
    nonisolated static func mock() -> ConnectionStatusObserver {
        ConnectionStatusObserver(connectionService: GemConnectionService(), monitors: [])
    }
}
