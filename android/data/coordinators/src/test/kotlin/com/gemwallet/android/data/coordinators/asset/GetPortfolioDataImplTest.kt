package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.blockchain.services.PerpetualService
import com.gemwallet.android.data.repositories.assets.CurrencyRatesService
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.model.Session
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.ChartValue
import com.wallet.core.primitives.ChartValuePercentage
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FiatRate
import com.wallet.core.primitives.PortfolioAssets
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
import uniffi.gemstone.GemPortfolioService
import com.gemwallet.android.serializer.toJson
import org.junit.Test

class GetPortfolioDataImplTest {

    private val portfolioService = mockk<GemPortfolioService>()
    private val currencyRatesService = mockk<CurrencyRatesService>(relaxed = true)
    private val perpetualService = mockk<PerpetualService>(relaxed = true)
    private val sessionRepository = mockk<SessionRepository>(relaxed = true)

    private val subject = GetPortfolioDataImpl(
        portfolioService = portfolioService,
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
    fun getPortfolioData_requestsCurrentWalletAssets_andConvertsSortedValues() = runTest {
        val wallet = mockWallet(id = "wallet-1")
        coEvery { sessionRepository.getCurrentWallet() } returns wallet
        every { currencyRatesService.getCurrencyRate(Currency.EUR) } returns flowOf(FiatRate(Currency.EUR, 2.0))
        coEvery { portfolioService.getWalletAssets(wallet.id.id, ChartPeriod.Day.toJson()) } returns portfolio().toJson()

        val result = subject.getPortfolioData(PortfolioType.Wallet, period = ChartPeriod.Day, currency = Currency.EUR)

        coVerify { portfolioService.getWalletAssets(wallet.id.id, ChartPeriod.Day.toJson()) }
        val values = result.charts.single().values
        assertEquals(listOf(1_000L, 2_000L), values.map { it.date })
        assertEquals(listOf(4.0, 6.0), values.map { it.value })
    }

    @Test
    fun getPortfolioData_convertsAllTimeStatisticsByRate() = runTest {
        coEvery { sessionRepository.getCurrentWallet() } returns mockWallet(id = "wallet-1")
        every { currencyRatesService.getCurrencyRate(Currency.EUR) } returns flowOf(FiatRate(Currency.EUR, 2.0))
        val allTimeHigh = ChartValuePercentage(date = 10L, value = 99f, percentage = 5f)
        val allTimeLow = ChartValuePercentage(date = 20L, value = 10f, percentage = -3f)
        coEvery { portfolioService.getWalletAssets(any(), any()) } returns
            portfolio(allTimeHigh = allTimeHigh, allTimeLow = allTimeLow).toJson()

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
