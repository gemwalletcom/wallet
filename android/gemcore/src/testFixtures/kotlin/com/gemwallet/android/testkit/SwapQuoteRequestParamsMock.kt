package com.gemwallet.android.testkit

import com.gemwallet.android.application.swap.cases.SwapQuoteRequestParams
import com.gemwallet.android.model.AssetInfo
import java.math.BigDecimal

fun mockSwapQuoteRequestParams(
    value: BigDecimal = BigDecimal.ONE,
    pay: AssetInfo = mockAssetInfo(asset = mockAssetSolana()),
    receive: AssetInfo = mockAssetInfo(asset = mockAssetSolanaUSDC()),
) = SwapQuoteRequestParams(
    value = value,
    pay = pay,
    receive = receive,
)
