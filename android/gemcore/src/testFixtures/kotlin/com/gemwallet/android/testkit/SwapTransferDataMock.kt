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
    minFromAmount: BigInteger? = null,
    toAmount: BigInteger = BigInteger.ONE,
    toAddress: String = from.address,
    provider: SwapProvider = SwapProvider.Hyperliquid,
    slippageBps: UInt = 50u,
    etaInSeconds: UInt? = null,
    useMaxAmount: Boolean = false,
) = SwapQuote(
    fromAddress = from.address,
    fromValue = fromAmount,
    minFromValue = minFromAmount,
    toAddress = toAddress,
    toValue = toAmount,
    providerData = SwapProviderData(provider = provider.toGem(), name = provider.string, protocolName = provider.string),
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
    memo: String? = null,
) = GemSwapTransfer(
    quote = mockSwapQuote(from = from, fromAmount = fromAmount, toAmount = toAmount, toAddress = toAddress, useMaxAmount = useMaxAmount),
    data = SwapQuoteData(
        to = toAddress,
        dataType = SwapQuoteDataType.CONTRACT,
        value = BigInteger.ZERO,
        data = "",
        memo = memo,
        approval = null,
        gasLimit = null,
    ),
    recipient = from.address,
    value = fromAmount,
    useMaxAmount = useMaxAmount,
)
