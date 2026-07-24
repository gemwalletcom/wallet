// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.AlienProvider
import struct Gemstone.GemServiceEndpoint
import class Gemstone.GemServiceStatus

public struct ServiceStatusService: ServiceStatusServiceable, Sendable {
    private let client: GemServiceStatus

    public init(provider: any AlienProvider) {
        self.client = GemServiceStatus(provider: provider)
    }

    public var endpoints: [GemServiceEndpoint] {
        client.getEndpoints()
    }

    public func endpointLatency(url: String) async throws -> UInt64 {
        try await client.getEndpointLatency(url: url)
    }
}
