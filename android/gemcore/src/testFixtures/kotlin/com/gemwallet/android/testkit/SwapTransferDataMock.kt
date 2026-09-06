package com.gemwallet.android.testkit

import com.gemwallet.android.domains.confirm.swap
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.TransactionInputType
import uniffi.gemstone.GemTransferData
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.SwapProvider
import com.gemwallet.android.ext.toGem
import uniffi.gemstone.ApprovalData
import uniffi.gemstone.GemSwapTransfer
import uniffi.gemstone.SwapData
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

fun mockSwapTransferData(
    from: Account = mockAccount(),
    fromAsset: Asset = mockAssetSolana(),
    fromAmount: BigInteger = BigInteger.ZERO,
    minFromAmount: BigInteger? = null,
    toAsset: Asset = mockAssetSolanaUSDC(),
    toAmount: BigInteger = BigInteger.ONE,
    approval: ApprovalData? = null,
    useMaxAmount: Boolean = false,
    toAddress: String = from.address,
    provider: SwapProvider = SwapProvider.Hyperliquid,
    dataType: SwapQuoteDataType = SwapQuoteDataType.TRANSFER,
) : GemTransferData {
    val swapData = SwapData(
        quote = mockSwapQuote(
            from = from,
            fromAmount = fromAmount,
            minFromAmount = minFromAmount,
            toAmount = toAmount,
            toAddress = toAddress,
            provider = provider,
            useMaxAmount = useMaxAmount,
        ),
        data = SwapQuoteData(
            to = toAddress,
            dataType = dataType,
            value = BigInteger.ZERO,
            data = "",
            memo = null,
            approval = approval,
            gasLimit = null,
        ),
    )
    return GemTransferData(
        inputType = TransactionInputType.swap(fromAsset, toAsset, swapData),
        recipient = GemRecipient(address = swapData.data.to, memo = swapData.data.memo),
        value = fromAmount,
        useMaxAmount = useMaxAmount,
        minimumValue = minFromAmount,
    )
}

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
