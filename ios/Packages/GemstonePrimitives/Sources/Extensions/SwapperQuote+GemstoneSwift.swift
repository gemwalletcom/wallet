// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import struct Gemstone.SwapperProviderType
import struct Gemstone.SwapperQuote
import Primitives

public extension Gemstone.SwapperQuote {
    func map() throws -> Primitives.SwapQuote {
        try Primitives.SwapQuote(
            fromAddress: request.walletAddress,
            fromValue: fromValue,
            toAddress: request.destinationAddress,
            toValue: toValue,
            providerData: data.provider.map(),
            slippageBps: data.slippageBps,
            etaInSeconds: etaInSeconds,
            useMaxAmount: request.options.useMaxAmount,
        )
    }

    var toValueBigInt: BigInt {
        (try? BigInt.from(string: toValue)) ?? .zero
    }

    var fromValueBigInt: BigInt {
        (try? BigInt.from(string: fromValue)) ?? .zero
    }
}

extension Gemstone.SwapperProviderType {
    func map() throws -> Primitives.SwapProviderData {
        try Primitives.SwapProviderData(
            provider: id.map(),
            name: name,
            protocolName: self.protocol,
        )
    }
}
