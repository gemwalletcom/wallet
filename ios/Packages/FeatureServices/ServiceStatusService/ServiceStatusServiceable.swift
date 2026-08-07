// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

public protocol ServiceStatusServiceable: Sendable {
    var endpoints: [ServiceEndpoint] { get }

    func endpointLatency(url: String) async throws -> UInt64
}
