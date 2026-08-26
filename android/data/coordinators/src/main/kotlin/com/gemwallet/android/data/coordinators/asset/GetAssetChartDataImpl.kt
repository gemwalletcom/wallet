package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.GetAssetChartData
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.assets.CurrencyRatesService
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.ChartValue
import com.wallet.core.primitives.Charts
import com.wallet.core.primitives.Currency
import kotlinx.coroutines.flow.firstOrNull
import uniffi.gemstone.GemChartService

class GetAssetChartDataImpl(
    private val chartService: GemChartService,
    private val assetsRepository: AssetsRepository,
    private val currencyRatesService: CurrencyRatesService,
) : GetAssetChartData {

    override suspend fun getAssetChartData(
        assetId: AssetId,
        period: ChartPeriod,
        currency: Currency,
    ): List<ChartValue> {
        val chart = chartService.getCharts(assetId.toIdentifier(), period.toJson()).decodeJson<Charts>()
        chart.market?.let {
            assetsRepository.updateAssetMarket(assetId, it, currency)
        }

        val rate = currencyRatesService.getCurrencyRate(currency).firstOrNull()?.rate?.toFloat() ?: return emptyList()
        return chart.prices
            .map { it.copy(value = it.value * rate) }
            .sortedBy { it.timestamp }
    }
}
