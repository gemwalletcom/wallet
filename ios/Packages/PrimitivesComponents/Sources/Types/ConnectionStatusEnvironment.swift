// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemConnectionService
import enum Gemstone.GemRefreshKind
import GemstonePrimitives
import Primitives
import SwiftUI

public extension EnvironmentValues {
    @Entry var connectionStatus: ConnectionStatus = .online
}

public extension ConnectionStatus {
    func refreshInterval(for kind: GemRefreshKind) -> TimeInterval {
        GemConnectionService.shared.refreshInterval(kind: kind, status: map())
    }
}
