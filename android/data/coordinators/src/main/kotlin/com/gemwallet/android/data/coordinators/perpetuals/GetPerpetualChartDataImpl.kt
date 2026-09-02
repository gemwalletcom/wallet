package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.cases.GetPerpetualChartData
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemPerpetualService
import com.gemwallet.android.ext.twoSubtokenIds
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.ChartCandleStick
import com.wallet.core.primitives.ChartPeriod
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

class GetPerpetualChartDataImpl(
    private val perpetualService: GemPerpetualService,
) : GetPerpetualChartData {

    override suspend fun getPerpetualChartData(
        assetId: AssetId,
        period: ChartPeriod
    ): List<ChartCandleStick> {
        val symbol = assetId.twoSubtokenIds()?.second ?: return emptyList()
        return withContext(Dispatchers.IO) {
            perpetualService.getCandlesticks(Chain.HyperCore.string, symbol, period.toJson()).map { it.decodeJson() }
        }
    }
}