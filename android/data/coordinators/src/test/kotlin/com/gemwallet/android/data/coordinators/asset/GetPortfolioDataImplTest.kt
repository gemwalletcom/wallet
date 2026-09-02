package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.application.session.cases.GetCurrentWallet
import com.gemwallet.android.model.Session
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PortfolioChartData
import com.wallet.core.primitives.PortfolioChartType
import com.wallet.core.primitives.PortfolioData
import com.wallet.core.primitives.PortfolioType
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemPortfolioDataInput
import uniffi.gemstone.GemPortfolioService

class GetPortfolioDataImplTest {

    private val portfolioService = mockk<GemPortfolioService>()
    private val getCurrentWallet = mockk<GetCurrentWallet>(relaxed = true)

    private val subject = GetPortfolioDataImpl(
        portfolioService = portfolioService,
        getCurrentWallet = getCurrentWallet,
    )

    @Test
    fun getPortfolioData_asksCoreForTheCurrentWallet() = runTest {
        val wallet = mockWallet(id = "wallet-1")
        val data = PortfolioData(
            charts = listOf(PortfolioChartData(chartType = PortfolioChartType.Value, values = emptyList())),
            statistics = emptyList(),
            availablePeriods = listOf(ChartPeriod.Day),
        )
        coEvery { getCurrentWallet.getCurrentWallet() } returns wallet
        coEvery {
            portfolioService.portfolioData(GemPortfolioDataInput.Wallet(wallet.id.id, ChartPeriod.Day.toJson(), Currency.EUR.toGem()))
        } returns data.toJson()

        val result = subject.getPortfolioData(PortfolioType.Wallet, period = ChartPeriod.Day, currency = Currency.EUR)

        assertEquals(data, result)
    }

    @Test
    fun getPortfolioData_asksCoreForThePerpetualAccount() = runTest {
        val address = "0xabc"
        val wallet = mockWallet(id = "wallet-1", accounts = listOf(com.gemwallet.android.testkit.mockAccount(chain = Chain.HyperCore, address = address)))
        val data = PortfolioData(charts = emptyList(), statistics = emptyList(), availablePeriods = emptyList())
        coEvery { getCurrentWallet.getCurrentWallet() } returns wallet
        coEvery {
            portfolioService.portfolioData(GemPortfolioDataInput.Perpetuals(Chain.HyperCore.string, address, ChartPeriod.All.toJson()))
        } returns data.toJson()

        val result = subject.getPortfolioData(PortfolioType.Perpetuals, period = ChartPeriod.All, currency = Currency.USD)

        assertEquals(data, result)
    }

    @Test(expected = IllegalStateException::class)
    fun getPortfolioData_perpetualsThrowsWithoutAccount() = runTest {
        coEvery { getCurrentWallet.getCurrentWallet() } returns null

        subject.getPortfolioData(PortfolioType.Perpetuals, period = ChartPeriod.All, currency = Currency.USD)
    }
}
