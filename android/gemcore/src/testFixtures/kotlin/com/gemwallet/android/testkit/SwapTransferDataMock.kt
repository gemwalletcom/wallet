package com.gemwallet.android.testkit

import com.wallet.core.primitives.Account
import com.wallet.core.primitives.SwapProvider
import com.gemwallet.android.ext.toGem
import uniffi.gemstone.GemSwapTransfer
import uniffi.gemstone.SwapProviderData
import uniffi.gemstone.SwapQuote
import uniffi.gemstone.SwapQuoteData
import uniffi.gemstone.SwapQuoteDataType
import java.math.BigInteger

fun mockSwapQuote(
    from: Account = mockAccount(),
    fromAmount: BigInteger = BigInteger.ZERO,
    toAmount: BigInteger = BigInteger.ONE,
    toAddress: String = from.address,
    slippageBps: UInt = 50u,
    etaInSeconds: UInt? = null,
    useMaxAmount: Boolean = false,
) = SwapQuote(
    fromAddress = from.address,
    fromValue = fromAmount,
    minFromValue = null,
    toAddress = toAddress,
    toValue = toAmount,
    providerData = SwapProviderData(provider = SwapProvider.Hyperliquid.toGem(), name = SwapProvider.Hyperliquid.string, protocolName = SwapProvider.Hyperliquid.string),
    slippageBps = slippageBps,
    etaInSeconds = etaInSeconds,
    useMaxAmount = useMaxAmount,
)

fun mockGemSwapTransfer(
    from: Account = mockAccount(),
    fromAmount: BigInteger = BigInteger.ZERO,
    toAmount: BigInteger = BigInteger.ONE,
    toAddress: String = from.address,
    useMaxAmount: Boolean = false,
) = GemSwapTransfer(
    quote = mockSwapQuote(from = from, fromAmount = fromAmount, toAmount = toAmount, toAddress = toAddress, useMaxAmount = useMaxAmount),
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
