// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemNameServiceProtocol
import Primitives

public extension GemNameServiceProtocol {
    func getNameRecord(name: String, chain: Chain) async throws -> NameRecord? {
        try await getNameRecord(name: name, chain: chain.rawValue).map { $0.map() }
    }
}
