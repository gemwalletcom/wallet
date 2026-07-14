// Copyright (c). Gem Wallet. All rights reserved.

import Network

struct NWPathMapper {
    private let vpnInterfacePrefixes = ["utun", "ipsec", "ppp", "tap", "tun"]
    private let transportTypes: [(NWInterface.InterfaceType, NetworkTransportType)] = [
        (.wifi, .wifi),
        (.cellular, .cellular),
        (.wiredEthernet, .wiredEthernet),
        (.loopback, .loopback),
        (.other, .other),
    ]

    func state(from path: NWPath) -> ConnectivityState {
        switch path.status {
        case .satisfied:
            .satisfied(NetworkPath(
                transports: Set(transportTypes.filter { path.usesInterfaceType($0.0) }.map(\.1)),
                isExpensive: path.isExpensive,
                isConstrained: path.isConstrained,
                isVPN: path.availableInterfaces.contains { interface in
                    vpnInterfacePrefixes.contains { interface.name.hasPrefix($0) }
                },
            ))
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
