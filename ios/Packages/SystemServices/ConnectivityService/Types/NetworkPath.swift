// Copyright (c). Gem Wallet. All rights reserved.

public struct NetworkPath: Equatable, Sendable {
    public let transports: Set<NetworkTransportType>
    public let isExpensive: Bool
    public let isConstrained: Bool
    public let isVPN: Bool

    public init(
        transports: Set<NetworkTransportType>,
        isExpensive: Bool,
        isConstrained: Bool,
        isVPN: Bool,
    ) {
        self.transports = transports
        self.isExpensive = isExpensive
        self.isConstrained = isConstrained
        self.isVPN = isVPN
    }
}
