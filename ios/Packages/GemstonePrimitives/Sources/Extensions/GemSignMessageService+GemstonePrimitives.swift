// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemSignMessagePreview
import protocol Gemstone.GemSignMessageServiceProtocol
import Primitives

public extension GemSignMessageServiceProtocol {
    func addressNames(chain: Chain, preview: GemSignMessagePreview) async -> [ChainAddress: AddressName] {
        let names = await addressNames(chain: chain.rawValue, preview: preview).compactMap { try? AddressName($0) }
        return Dictionary(uniqueKeysWithValues: names.map { (ChainAddress(chain: $0.chain, address: $0.address), $0) })
    }
}
