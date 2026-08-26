// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemNameServiceProtocol
import Primitives

public extension GemNameServiceProtocol {
    func addressNames(requests: [Primitives.ChainAddress]) async throws -> [Primitives.ChainAddress: Primitives.AddressName] {
        let names = try await getAddressNames(requests: requests.map { try $0.json() }).map { try Primitives.AddressName($0) }
        return Dictionary(uniqueKeysWithValues: names.map { (Primitives.ChainAddress(chain: $0.chain, address: $0.address), $0) })
    }
}
