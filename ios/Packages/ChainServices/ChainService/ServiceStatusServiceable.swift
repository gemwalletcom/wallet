// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemServiceEndpoint

public protocol ServiceStatusServiceable: Sendable {
    var endpoints: [GemServiceEndpoint] { get }

    func endpointLatency(url: String) async throws -> UInt64
}
