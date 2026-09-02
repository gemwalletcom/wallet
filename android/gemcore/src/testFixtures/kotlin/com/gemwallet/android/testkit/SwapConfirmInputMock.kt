package com.gemwallet.android.testkit

import com.gemwallet.android.domains.confirm.confirmInput
import com.gemwallet.android.domains.confirm.swap
import uniffi.gemstone.GemConfirmInput
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.GemTransactionInputType
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

fun mockSwapParams(
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
) : GemConfirmInput {
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
        inputType = GemTransactionInputType.swap(fromAsset, toAsset, swapData),
        recipient = GemRecipient(address = swapData.data.to, memo = swapData.data.memo),
        value = fromAmount.toString(),
        useMaxAmount = useMaxAmount,
        minimumValue = minFromAmount?.toString(),
    ).confirmInput(from)
}
