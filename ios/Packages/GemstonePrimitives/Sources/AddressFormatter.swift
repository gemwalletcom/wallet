// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import Primitives

public struct AddressFormatter {
    public enum Style {
        case short
        case full
        case extra(Int)
    }

    private let style: Self.Style
    private let address: String
    private let chain: Primitives.Chain?

    public init(
        style: Self.Style = .short,
        address: String,
        chain: Primitives.Chain?,
    ) {
        self.style = style
        self.address = address
        self.chain = chain
    }

    public func value() -> String {
        let gemstoneStyle: Gemstone.GemAddressFormatStyle = switch style {
        case .short: .short
        case .full: .full
        case let .extra(extra): .extra(extra: UInt32(clamping: max(extra, 0)))
        }

        return Gemstone.formatAddress(
            address: address,
            chain: chain?.rawValue,
            style: gemstoneStyle,
        )
    }
}
