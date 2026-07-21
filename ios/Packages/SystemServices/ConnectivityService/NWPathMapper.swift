// Copyright (c). Gem Wallet. All rights reserved.

import Network

struct NWPathMapper {
    func state(from path: NWPath) -> ConnectivityState {
        switch path.status {
        case .satisfied:
            .satisfied
        case .unsatisfied:
            .unsatisfied(reason(from: path.unsatisfiedReason))
        case .requiresConnection:
            .unknown
        @unknown default:
            .unknown
        }
    }

    private func reason(from reason: NWPath.UnsatisfiedReason) -> UnavailableReason {
        switch reason {
        case .notAvailable: .noNetwork
        case .cellularDenied: .cellularDenied
        case .wifiDenied: .wifiDenied
        case .localNetworkDenied: .localNetworkDenied
        case .vpnInactive: .vpnInactive
        @unknown default: .noNetwork
        }
    }
}
