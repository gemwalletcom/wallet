package com.gemwallet.android.application.assets.coordinators

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.ChartValue
import com.wallet.core.primitives.Currency

interface GetAssetChartData {
    suspend fun getAssetChartData(
        assetId: AssetId,
        period: ChartPeriod,
        currency: Currency,
    ): List<ChartValue>
}
