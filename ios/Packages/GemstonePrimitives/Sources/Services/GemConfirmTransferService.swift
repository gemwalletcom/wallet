// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemConfirmTransferServiceProtocol
import struct Gemstone.GemConfirmSimulationState
import Primitives

public extension GemConfirmTransferServiceProtocol {
    func explorerLink(chain: Primitives.Chain, address: String) -> BlockExplorerLink {
        addressUrl(chain: chain.rawValue, address: address).map()
    }
}

public extension GemConfirmSimulationState {
    var names: [Primitives.ChainAddress: Primitives.AddressName] {
        Dictionary(
            addressNames
                .map { $0.map() }
                .map { (Primitives.ChainAddress(chain: $0.chain, address: $0.address), $0) },
            uniquingKeysWith: { first, _ in first },
        )
    }
}
