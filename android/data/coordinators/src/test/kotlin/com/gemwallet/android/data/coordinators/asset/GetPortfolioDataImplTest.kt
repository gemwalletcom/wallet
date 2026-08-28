package com.gemwallet.android.data.coordinators.asset

import uniffi.gemstone.GemPerpetualService
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.model.Session
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.ChartDateValue
import com.wallet.core.primitives.ChartValuePercentage
import com.wallet.core.primitives.Currency
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
import uniffi.gemstone.GemPortfolioValues
import com.gemwallet.android.serializer.toJson
import org.junit.Test

class GetPortfolioDataImplTest {

    private val portfolioService = mockk<GemPortfolioService>()
    private val perpetualService = mockk<GemPerpetualService>(relaxed = true)
    private val sessionRepository = mockk<SessionRepository>(relaxed = true)

    private val subject = GetPortfolioDataImpl(
        portfolioService = portfolioService,
        perpetualService = perpetualService,
        sessionRepository = sessionRepository,
    )

    @Test
    fun getPortfolioData_usesCoreConvertedWalletValues() = runTest {
        val wallet = mockWallet(id = "wallet-1")
        val allTimeHigh = ChartValuePercentage(date = 10L, value = 198f, percentage = 5f)
        coEvery { sessionRepository.getCurrentWallet() } returns wallet
        coEvery { portfolioService.syncWalletValues(wallet.id.id, ChartPeriod.Day.toJson(), Currency.EUR.toJson()) } returns GemPortfolioValues(
            values = listOf(ChartDateValue(date = 1_000L, value = 4.0), ChartDateValue(date = 2_000L, value = 6.0)).map { it.toJson() },
            allTimeHigh = allTimeHigh.toJson(),
            allTimeLow = null,
        )

        val result = subject.getPortfolioData(PortfolioType.Wallet, period = ChartPeriod.Day, currency = Currency.EUR)

        val values = result.charts.single().values
        assertEquals(listOf(1_000L, 2_000L), values.map { it.date })
        assertEquals(listOf(4.0, 6.0), values.map { it.value })
        assertEquals(listOf(PortfolioStatistic.AllTimeHigh(allTimeHigh)), result.statistics)
    }

    @Test(expected = IllegalStateException::class)
    fun getPortfolioData_perpetualsThrowsWithoutAccount() = runTest {
        every { sessionRepository.session() } returns MutableStateFlow<Session?>(null)

        subject.getPortfolioData(PortfolioType.Perpetuals, period = ChartPeriod.All, currency = Currency.USD)
    }
}
