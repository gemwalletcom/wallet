// Copyright (c). Gem Wallet. All rights reserved.

public enum NetworkTransportType: String, Equatable, Sendable, CaseIterable {
    case wifi
    case cellular
    case wiredEthernet
    case loopback
    case other
}
