package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.cases.GetAssetChartData
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.ChartDateValue
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.ChartValue
import com.wallet.core.primitives.Currency
import uniffi.gemstone.GemChartService
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

class GetAssetChartDataImpl(
    private val chartService: GemChartService,
) : GetAssetChartData {

    override suspend fun getAssetChartData(
        assetId: AssetId,
        period: ChartPeriod,
        currency: Currency,
    ): List<ChartValue> {
        return withContext(Dispatchers.IO) {
            chartService.syncCharts(assetId.toIdentifier(), period.toJson(), currency.toJson())
                .map { it.decodeJson<ChartDateValue>() }
                .map { ChartValue(timestamp = (it.date / 1000).toInt(), value = it.value.toFloat()) }
        }
    }
}
