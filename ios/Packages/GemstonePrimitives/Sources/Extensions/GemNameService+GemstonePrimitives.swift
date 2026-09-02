// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemNameServiceProtocol
import Primitives

public extension GemNameServiceProtocol {
    func getNameRecord(name: String, chain: Primitives.Chain) async throws -> Primitives.NameRecord? {
        try await getNameRecord(name: name, chain: chain.rawValue).map { try Primitives.NameRecord($0) }
    }
}
