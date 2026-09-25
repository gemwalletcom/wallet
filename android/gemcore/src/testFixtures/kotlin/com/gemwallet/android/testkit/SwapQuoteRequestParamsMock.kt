package com.gemwallet.android.testkit

import com.gemwallet.android.application.swap.cases.SwapQuoteRequestParams
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.Crypto
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemSwapQuoteInput
import uniffi.gemstone.GemSwapRequest
import java.math.BigDecimal

fun mockSwapQuoteRequestParams(
    value: BigDecimal = BigDecimal.ONE,
    pay: AssetInfo = mockAssetInfo(asset = mockAsset(id = mockAssetId(chain = Chain.Solana), name = "Solana", symbol = "SOL", decimals = 9)),
    receive: AssetInfo = mockAssetInfo(asset = mockAsset(id = mockAssetId(chain = Chain.Solana, tokenId = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), name = "USD Coin", symbol = "USDC", decimals = 6, type = AssetType.SPL)),
) = SwapQuoteRequestParams(
    input = GemSwapQuoteInput(
        request = GemSwapRequest(
            payAssetId = pay.id().toIdentifier(),
            receiveAssetId = receive.id().toIdentifier(),
            value = Crypto(value, pay.asset.decimals).atomicValue,
            slippageBps = null,
        ),
        useMaxAmount = false,
    ),
    pay = pay,
    receive = receive,
)
