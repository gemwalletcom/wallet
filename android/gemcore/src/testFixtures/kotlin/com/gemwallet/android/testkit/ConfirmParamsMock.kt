package com.gemwallet.android.testkit

import com.gemwallet.android.model.ConfirmParams
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
) = ConfirmParams.SwapParams(
    from = from,
    fromAsset = fromAsset,
    toAsset = toAsset,
    swapData = SwapData(
        quote = SwapQuote(
            fromAddress = from.address,
            fromValue = fromAmount.toString(),
            minFromValue = minFromAmount?.toString(),
            toAddress = toAddress,
            toValue = toAmount.toString(),
            providerData = SwapProviderData(provider = provider, name = provider.string, protocolName = provider.string),
            slippageBps = 50u,
            etaInSeconds = null,
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
    ),
    amount = fromAmount,
    useMaxAmount = useMaxAmount,
)
