// Copyright (c). Gem Wallet. All rights reserved.

public enum ConnectivityState: Equatable, Sendable {
    case unknown
    case satisfied(NetworkPath)
    case unsatisfied(UnavailableReason)

    public var isOffline: Bool {
        if case .unsatisfied = self { return true }
        return false
    }
}
