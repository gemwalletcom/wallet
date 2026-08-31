package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.testkit.mockAsset
import com.wallet.core.primitives.ChartDateValue
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.ChartValue
import com.wallet.core.primitives.Currency
import io.mockk.coEvery
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemChartService

class GetAssetChartDataImplTest {

    private val chartService = mockk<GemChartService>()
    private val subject = GetAssetChartDataImpl(chartService = chartService)

    @Test
    fun getAssetChartData_mapsConvertedCoreValuesToChartValues() = runTest {
        val asset = mockAsset()
        val converted = listOf(
            ChartDateValue(date = 1_000L, value = 4.0),
            ChartDateValue(date = 2_000L, value = 6.0),
        )
        coEvery { chartService.syncCharts("bitcoin", ChartPeriod.Day.toJson(), Currency.EUR.toJson()) } returns converted.map { it.toJson() }

        val result = subject.getAssetChartData(
            assetId = asset.id,
            period = ChartPeriod.Day,
            currency = Currency.EUR,
        )

        assertEquals(listOf(ChartValue(timestamp = 1, value = 4.0f), ChartValue(timestamp = 2, value = 6.0f)), result)
    }
}
