// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemAddressService
import enum Gemstone.GemAddressFormatStyle
import Primitives

private let addressService = GemAddressService()

public struct AddressFormatter {
    private let style: GemAddressFormatStyle
    private let address: String
    private let chain: Primitives.Chain?

    public init(
        style: GemAddressFormatStyle = .short,
        address: String,
        chain: Primitives.Chain?,
    ) {
        self.style = style
        self.address = address
        self.chain = chain
    }

    public func value() -> String {
        addressService.format(address: address, chain: chain?.rawValue, style: style)
    }
}
