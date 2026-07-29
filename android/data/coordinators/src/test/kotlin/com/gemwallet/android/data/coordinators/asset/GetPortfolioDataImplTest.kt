package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.blockchain.services.PerpetualService
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.assets.CurrencyRatesService
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.model.Session
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetInfo
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.ChartValue
import com.wallet.core.primitives.ChartValuePercentage
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FiatRate
import com.wallet.core.primitives.PortfolioAsset
import com.wallet.core.primitives.PortfolioAssets
import com.wallet.core.primitives.PortfolioAssetsRequest
import com.wallet.core.primitives.PortfolioStatistic
import com.wallet.core.primitives.PortfolioType
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test

class GetPortfolioDataImplTest {

    private val gemDeviceApiClient = mockk<GemDeviceApiClient>()
    private val assetsRepository = mockk<AssetsRepository>(relaxed = true)
    private val currencyRatesService = mockk<CurrencyRatesService>(relaxed = true)
    private val perpetualService = mockk<PerpetualService>(relaxed = true)
    private val sessionRepository = mockk<SessionRepository>(relaxed = true)

    private val subject = GetPortfolioDataImpl(
        gemDeviceApiClient = gemDeviceApiClient,
        assetsRepository = assetsRepository,
        currencyRatesService = currencyRatesService,
        perpetualService = perpetualService,
        sessionRepository = sessionRepository,
    )

    private fun portfolio(
        allTimeHigh: ChartValuePercentage? = null,
        allTimeLow: ChartValuePercentage? = null,
    ) = PortfolioAssets(
        totalValue = 5.0f,
        values = listOf(
            ChartValue(timestamp = 2, value = 3.0f),
            ChartValue(timestamp = 1, value = 2.0f),
        ),
        allTimeHigh = allTimeHigh,
        allTimeLow = allTimeLow,
        allocation = emptyList(),
    )

    @Test
    fun getPortfolioData_requestsOnlyHeldAssets_andConvertsSortedValues() = runTest {
        val bitcoin = mockAsset()
        val ethereum = mockAssetEthereum()
        val held = mockAssetInfo(asset = bitcoin, balance = AssetBalance.create(bitcoin, available = "1000"))
        val empty = mockAssetInfo(asset = ethereum, balance = AssetBalance.create(ethereum))
        every { assetsRepository.getAssetsInfo() } returns flowOf(listOf(held, empty))
        every { currencyRatesService.getCurrencyRate(Currency.EUR) } returns flowOf(FiatRate("EUR", 2.0))
        coEvery { gemDeviceApiClient.getPortfolioAssets("day", any()) } returns portfolio()

        val result = subject.getPortfolioData(PortfolioType.Wallet, period = ChartPeriod.Day, currency = Currency.EUR)

        coVerify {
            gemDeviceApiClient.getPortfolioAssets(
                "day",
                PortfolioAssetsRequest(assets = listOf(PortfolioAsset(assetId = bitcoin.id, value = "1000"))),
            )
        }
        val values = result.charts.single().values
        assertEquals(listOf(1_000L, 2_000L), values.map { it.date })
        assertEquals(listOf(4.0, 6.0), values.map { it.value })
    }

    @Test
    fun getPortfolioData_convertsAllTimeStatisticsByRate() = runTest {
        val bitcoin = mockAsset()
        every { assetsRepository.getAssetsInfo() } returns
            flowOf(listOf(mockAssetInfo(asset = bitcoin, balance = AssetBalance.create(bitcoin, available = "1"))))
        every { currencyRatesService.getCurrencyRate(Currency.EUR) } returns flowOf(FiatRate("EUR", 2.0))
        val allTimeHigh = ChartValuePercentage(date = 10L, value = 99f, percentage = 5f)
        val allTimeLow = ChartValuePercentage(date = 20L, value = 10f, percentage = -3f)
        coEvery { gemDeviceApiClient.getPortfolioAssets(any(), any()) } returns
            portfolio(allTimeHigh = allTimeHigh, allTimeLow = allTimeLow)

        val result = subject.getPortfolioData(PortfolioType.Wallet, period = ChartPeriod.Day, currency = Currency.EUR)

        assertEquals(
            listOf(
                PortfolioStatistic.AllTimeHigh(allTimeHigh.copy(value = 198f)),
                PortfolioStatistic.AllTimeLow(allTimeLow.copy(value = 20f)),
            ),
            result.statistics,
        )
    }

    @Test(expected = IllegalStateException::class)
    fun getPortfolioData_perpetualsThrowsWithoutAccount() = runTest {
        every { sessionRepository.session() } returns MutableStateFlow<Session?>(null)

        subject.getPortfolioData(PortfolioType.Perpetuals, period = ChartPeriod.All, currency = Currency.USD)
    }

    @Test(expected = IllegalStateException::class)
    fun getPortfolioData_throwsWhenRateMissing() = runTest {
        every { currencyRatesService.getCurrencyRate(Currency.EUR) } returns flowOf(null)

        subject.getPortfolioData(PortfolioType.Wallet, period = ChartPeriod.Day, currency = Currency.EUR)
    }
}
