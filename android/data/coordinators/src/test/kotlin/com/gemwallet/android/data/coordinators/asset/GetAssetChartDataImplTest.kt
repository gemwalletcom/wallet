package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.data.repositories.assets.CurrencyRatesService
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetMarket
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.ChartValue
import com.wallet.core.primitives.Charts
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FiatRate
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import uniffi.gemstone.GemChartService
import uniffi.gemstone.GemPriceService
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class GetAssetChartDataImplTest {

    private val chartService = mockk<GemChartService>()
    private val priceService = mockk<GemPriceService>(relaxed = true)
    private val currencyRatesService = mockk<CurrencyRatesService>(relaxed = true)

    private val subject = GetAssetChartDataImpl(
        chartService = chartService,
        priceService = priceService,
        currencyRatesService = currencyRatesService,
    )

    @Test
    fun getAssetChartData_updatesMarketAndConvertsPrices() = runTest {
        val asset = mockAsset()
        val market = mockAssetMarket(marketCap = 1_000.0)
        val chart = Charts(
            market = market,
            prices = listOf(
                ChartValue(timestamp = 2, value = 3.0f),
                ChartValue(timestamp = 1, value = 2.0f),
            ),
            marketCaps = emptyList(),
            totalVolumes = emptyList(),
        )
        coEvery { chartService.getCharts("bitcoin", ChartPeriod.Day.toJson()) } returns chart.toJson()
        every { currencyRatesService.getCurrencyRate(Currency.EUR) } returns flowOf(FiatRate(Currency.EUR, 2.0))

        val result = subject.getAssetChartData(
            assetId = asset.id,
            period = ChartPeriod.Day,
            currency = Currency.EUR,
        )

        assertEquals(listOf(4.0f, 6.0f), result.map { it.value })
        coVerify { priceService.updateMarket("bitcoin", market.toJson(), Currency.EUR.toJson()) }
    }

    @Test
    fun getAssetChartData_returnsEmptyWhenRateMissing() = runTest {
        val asset = mockAsset()
        val chart = Charts(
            prices = listOf(ChartValue(timestamp = 1, value = 2.0f)),
            marketCaps = emptyList(),
            totalVolumes = emptyList(),
        )
        coEvery { chartService.getCharts("bitcoin", ChartPeriod.Day.toJson()) } returns chart.toJson()
        every { currencyRatesService.getCurrencyRate(Currency.EUR) } returns flowOf(null)

        val result = subject.getAssetChartData(
            assetId = asset.id,
            period = ChartPeriod.Day,
            currency = Currency.EUR,
        )

        assertTrue(result.isEmpty())
        coVerify(exactly = 0) { priceService.updateMarket(any(), any(), any()) }
    }
}
