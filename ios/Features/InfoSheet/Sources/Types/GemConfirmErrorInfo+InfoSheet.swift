// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmErrorInfo
import struct Gemstone.GemFormattedNumber
import GemstonePrimitives
import Primitives

extension GemConfirmErrorInfo {
    var networkName: String {
        asset.map { $0.toPrimitives().chain.networkName } ?? title
    }

    /// The amount a sheet asks for, with the fiat it is worth when a price is known.
    var requiredWithFiat: String {
        guard let amount = required?.text() else { return .empty }
        guard let fiat = requiredFiat?.text() else { return amount }
        return "\(amount) (~\(fiat))"
    }
}

extension GemFormattedNumber? {
    var boldText: String {
        self?.text().boldMarkdown() ?? .empty
    }
}
