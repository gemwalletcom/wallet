package com.gemwallet.android.testkit

import com.gemwallet.android.model.ConfirmParams
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemSwapQuoteDataType
import uniffi.gemstone.SwapperProvider
import java.math.BigInteger

fun mockSwapParams(
    from: Account = mockAccount(),
    fromAsset: Asset = mockAssetSolana(),
    fromAmount: BigInteger = BigInteger.ZERO,
    minFromAmount: BigInteger? = null,
    toAsset: Asset = mockAssetSolanaUSDC(),
    toAmount: BigInteger = BigInteger.ONE,
    useMaxAmount: Boolean = false,
) = ConfirmParams.SwapParams(
    from = from,
    fromAsset = fromAsset,
    fromAmount = fromAmount,
    minFromAmount = minFromAmount,
    toAsset = toAsset,
    toAmount = toAmount,
    swapData = "",
    memo = null,
    providerId = SwapperProvider.HYPERLIQUID,
    providerName = "Hyperliquid",
    protocol = "Hyperliquid",
    protocolId = "hyperliquid",
    toAddress = from.address,
    value = "0",
    slippageBps = 50u,
    etaInSeconds = null,
    dataType = GemSwapQuoteDataType.TRANSFER,
    useMaxAmount = useMaxAmount,
)
