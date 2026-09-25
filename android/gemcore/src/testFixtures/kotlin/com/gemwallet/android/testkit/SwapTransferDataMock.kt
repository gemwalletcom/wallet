package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.SwapProvider
import uniffi.gemstone.GemSwapTransfer
import uniffi.gemstone.SwapProviderData
import uniffi.gemstone.SwapQuote
import uniffi.gemstone.SwapQuoteData
import uniffi.gemstone.SwapQuoteDataType
import java.math.BigInteger

fun mockGemSwapTransfer(from: Account = mockAccount(), fromAmount: BigInteger = BigInteger.ZERO, toAmount: BigInteger = BigInteger.ONE, toAddress: String = from.address, useMaxAmount: Boolean = false) = GemSwapTransfer(
    quote = mockSwapQuote(
        fromAddress = from.address,
        fromValue = fromAmount,
        toAddress = toAddress,
        toValue = toAmount,
        providerData = mockSwapProviderData(provider = SwapProvider.Hyperliquid.toGem(), name = SwapProvider.Hyperliquid.string, protocolName = SwapProvider.Hyperliquid.string),
        slippageBps = 50u,
        useMaxAmount = useMaxAmount,
    ),
    data = SwapQuoteData(
        to = toAddress,
        dataType = SwapQuoteDataType.CONTRACT,
        value = BigInteger.ZERO,
        data = "",
        memo = null,
        approval = null,
        gasLimit = null,
    ),
    recipient = from.address,
    value = fromAmount,
    useMaxAmount = useMaxAmount,
)
