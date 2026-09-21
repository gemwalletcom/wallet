package com.gemwallet.android.testkit

import com.gemwallet.android.application.swap.cases.SwapQuoteRequestParams
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.Crypto
import uniffi.gemstone.GemSwapQuoteInput
import uniffi.gemstone.GemSwapRequest
import java.math.BigDecimal

fun mockSwapQuoteRequestParams(value: BigDecimal = BigDecimal.ONE, pay: AssetInfo = mockAssetInfo(asset = mockAssetSolana()), receive: AssetInfo = mockAssetInfo(asset = mockAssetSolanaUSDC())) = SwapQuoteRequestParams(
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
