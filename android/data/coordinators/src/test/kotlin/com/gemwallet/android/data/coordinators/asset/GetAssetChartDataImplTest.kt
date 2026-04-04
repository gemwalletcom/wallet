package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.services.gemapi.GemApiClient
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
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class GetAssetChartDataImplTest {

    private val gemApiClient = mockk<GemApiClient>()
    private val assetsRepository = mockk<AssetsRepository>(relaxed = true)

    private val subject = GetAssetChartDataImpl(
        gemApiClient = gemApiClient,
        assetsRepository = assetsRepository,
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
        coEvery { gemApiClient.getChart("bitcoin", "day") } returns chart
        every { assetsRepository.getCurrencyRate(Currency.EUR) } returns flowOf(FiatRate("EUR", 2.0))

        val result = subject.getAssetChartData(
            assetId = asset.id,
            period = ChartPeriod.Day,
            currency = Currency.EUR,
        )

        assertEquals(listOf(4.0f, 6.0f), result.map { it.value })
        coVerify { assetsRepository.updateAssetMarket(asset.id, market, Currency.EUR) }
    }

    @Test
    fun getAssetChartData_returnsEmptyWhenRateMissing() = runTest {
        val asset = mockAsset()
        val chart = Charts(
            prices = listOf(ChartValue(timestamp = 1, value = 2.0f)),
            marketCaps = emptyList(),
            totalVolumes = emptyList(),
        )
        coEvery { gemApiClient.getChart("bitcoin", "day") } returns chart
        every { assetsRepository.getCurrencyRate(Currency.EUR) } returns flowOf(null)

        val result = subject.getAssetChartData(
            assetId = asset.id,
            period = ChartPeriod.Day,
            currency = Currency.EUR,
        )

        assertTrue(result.isEmpty())
        coVerify(exactly = 0) { assetsRepository.updateAssetMarket(any(), any(), any()) }
    }
}
