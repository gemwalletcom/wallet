// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import struct Gemstone.SwapperQuote
import enum Gemstone.SwapProvider
import struct Gemstone.SwapProviderData
import struct Gemstone.SwapQuote

public extension SwapperQuote {
    var swapQuote: SwapQuote {
        SwapQuote(
            fromAddress: request.walletAddress,
            fromValue: fromValue,
            minFromValue: minFromValue,
            toAddress: request.destinationAddress,
            toValue: toValue,
            providerData: SwapProviderData(provider: data.provider.id, name: data.provider.name, protocolName: data.provider.protocol),
            slippageBps: data.slippageBps,
            etaInSeconds: etaInSeconds,
            useMaxAmount: request.options.useMaxAmount,
        )
    }

    static func mock(
        fromValue: BigUInt = 1_000_000_000_000_000_000,
        minFromValue: BigUInt? = nil,
        toValue: BigUInt = 250_000_000_000,
        provider: SwapProvider = .pancakeswapV3,
        etaInSeconds: UInt32? = nil,
    ) -> SwapperQuote {
        SwapperQuote(
            fromValue: fromValue,
            minFromValue: minFromValue,
            toValue: toValue,
            data: .mock(provider: provider),
            request: .mock(),
            etaInSeconds: etaInSeconds,
        )
    }
}
