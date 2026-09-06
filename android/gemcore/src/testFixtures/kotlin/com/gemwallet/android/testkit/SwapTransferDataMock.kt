package com.gemwallet.android.testkit

import com.gemwallet.android.domains.confirm.swap
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.TransactionInputType
import uniffi.gemstone.GemTransferData
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.swap.ApprovalData
import com.wallet.core.primitives.SwapProvider
import com.wallet.core.primitives.swap.SwapProviderData
import com.wallet.core.primitives.swap.SwapQuoteData
import com.wallet.core.primitives.swap.SwapQuote
import com.wallet.core.primitives.swap.SwapData
import com.wallet.core.primitives.swap.SwapQuoteDataType
import com.gemwallet.android.serializer.toJson
import uniffi.gemstone.GemSwapTransfer
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
    fromValue = fromAmount.toString(),
    minFromValue = minFromAmount?.toString(),
    toAddress = toAddress,
    toValue = toAmount.toString(),
    providerData = SwapProviderData(provider = provider, name = provider.string, protocolName = provider.string),
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
    dataType: SwapQuoteDataType = SwapQuoteDataType.Transfer,
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
            value = "0",
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
    quote = mockSwapQuote(from = from, fromAmount = fromAmount, toAmount = toAmount, toAddress = toAddress, useMaxAmount = useMaxAmount).toJson(),
    data = SwapQuoteData(
        to = toAddress,
        dataType = SwapQuoteDataType.Contract,
        value = "0",
        data = "",
        memo = memo,
        approval = null,
        gasLimit = null,
    ).toJson(),
    recipient = from.address,
    value = fromAmount,
    useMaxAmount = useMaxAmount,
)
