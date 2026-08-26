// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemNameServiceProtocol
import GemstonePrimitives
import Primitives

public struct GemstoneNameService: NameServiceable {
    private let service: any GemNameServiceProtocol

    public init(service: any GemNameServiceProtocol) {
        self.service = service
    }

    public func getName(name: String, chain: String) async throws -> NameRecord? {
        try await service.resolve(name: name, chain: chain).map { try NameRecord($0) }
    }
}
