// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemAddressFormatStyle
import protocol Gemstone.GemAddressServiceProtocol
import Primitives

public extension GemAddressServiceProtocol {
    func format(address: String, chain: Primitives.Chain?, style: GemAddressFormatStyle = .short) -> String {
        format(address: address, chain: chain?.map(), style: style)
    }
}
