// Copyright (c). Gem Wallet. All rights reserved.

public enum UnavailableReason: Equatable, Sendable {
    case noNetwork
    case cellularDenied
    case wifiDenied
    case localNetworkDenied
    case vpnInactive
}
