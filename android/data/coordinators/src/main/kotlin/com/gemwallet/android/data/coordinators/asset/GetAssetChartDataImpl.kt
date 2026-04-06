package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.GetAssetChartData
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.services.gemapi.GemApiClient
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.ChartValue
import com.wallet.core.primitives.Currency
import kotlinx.coroutines.flow.firstOrNull

class GetAssetChartDataImpl(
    private val gemApiClient: GemApiClient,
    private val assetsRepository: AssetsRepository,
) : GetAssetChartData {

    override suspend fun getAssetChartData(
        assetId: AssetId,
        period: ChartPeriod,
        currency: Currency,
    ): List<ChartValue> {
        val chart = gemApiClient.getChart(assetId.toIdentifier(), period.string)
        chart.market?.let {
            assetsRepository.updateAssetMarket(assetId, it, currency)
        }

        val rate = assetsRepository.getCurrencyRate(currency).firstOrNull()?.rate?.toFloat() ?: return emptyList()
        return chart.prices
            .map { it.copy(value = it.value * rate) }
            .sortedBy { it.timestamp }
    }
}
