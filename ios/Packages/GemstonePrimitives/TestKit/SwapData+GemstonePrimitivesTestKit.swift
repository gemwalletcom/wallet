// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.ApprovalData
import struct Gemstone.ContractCallData
import enum Gemstone.SwapProvider
import struct Gemstone.SwapData
import struct Gemstone.SwapProviderData
import struct Gemstone.SwapQuote
import struct Gemstone.SwapQuoteData
import BigInt
import Foundation

public extension SwapQuote {
    static func mock(
        fromValue: BigUInt = 1_000_000_000_000_000_000,
        minFromValue: BigUInt? = nil,
        toValue: BigUInt = 2_000_000_000_000_000_000,
        providerData: SwapProviderData = .mock(),
        walletAddress: String = "0x0000000000000000000000000000000000000000",
        etaInSeconds: UInt32 = 123,
        useMaxAmount: Bool = false,
    ) -> SwapQuote {
        SwapQuote(
            fromAddress: walletAddress,
            fromValue: fromValue,
            minFromValue: minFromValue,
            toAddress: walletAddress,
            toValue: toValue,
            providerData: providerData,
            slippageBps: 50,
            etaInSeconds: etaInSeconds,
            useMaxAmount: useMaxAmount,
        )
    }
}

public extension SwapProviderData {
    static func mock(
        provider: SwapProvider = .mayan,
        name: String = "Uniswap",
        protocolName: String = "Uniswap v3",
    ) -> SwapProviderData {
        SwapProviderData(provider: provider, name: name, protocolName: protocolName)
    }
}

public extension SwapQuoteData {
    static func mock(
        approval: ApprovalData? = nil,
        gasLimit: String? = "",
    ) -> SwapQuoteData {
        SwapQuoteData(
            to: "",
            dataType: .contract,
            value: .zero,
            data: "",
            memo: nil,
            approval: approval,
            gasLimit: gasLimit,
        )
    }
}

public extension ApprovalData {
    static func mock() -> ApprovalData {
        ApprovalData(token: "", spender: "", value: .zero, isUnlimited: false)
    }
}

public extension SwapData {
    static func mock(
        quote: SwapQuote = .mock(),
        data: SwapQuoteData = .mock(),
    ) -> SwapData {
        SwapData(quote: quote, data: data)
    }
}

public extension ContractCallData {
    static func mock(
        contractAddress: String = "",
        callData: String = "",
        approval: ApprovalData? = nil,
        gasLimit: String? = nil,
    ) -> ContractCallData {
        ContractCallData(
            contractAddress: contractAddress,
            callData: callData,
            approval: approval,
            gasLimit: gasLimit,
        )
    }
}
