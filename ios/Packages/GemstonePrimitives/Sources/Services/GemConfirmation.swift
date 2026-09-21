// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemConfirmationProtocol
import struct Gemstone.GemConfirmSimulationState
import Primitives

public extension GemConfirmationProtocol {
    var currency: Primitives.Currency {
        getCurrency().toPrimitives()
    }

    func explorerLink(chain: Primitives.Chain, address: String) -> BlockExplorerLink {
        addressUrl(chain: chain.rawValue, address: address).toPrimitives()
    }
}
