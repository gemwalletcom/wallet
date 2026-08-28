package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.GetPortfolioData
import com.gemwallet.android.application.assets.coordinators.walletChartPeriods
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemPerpetualService
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.hyperliquidAccount
import com.wallet.core.primitives.ChartDateValue
import com.wallet.core.primitives.ChartValuePercentage
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualPortfolio
import com.wallet.core.primitives.PerpetualPortfolioTimeframeData
import com.wallet.core.primitives.PortfolioChartData
import com.wallet.core.primitives.PortfolioChartType
import com.wallet.core.primitives.PortfolioData
import com.wallet.core.primitives.PortfolioMarginUsage
import com.wallet.core.primitives.PortfolioStatistic
import com.wallet.core.primitives.PortfolioType
import kotlinx.coroutines.flow.firstOrNull
import uniffi.gemstone.GemPortfolioService
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson

class GetPortfolioDataImpl(
    private val portfolioService: GemPortfolioService,
    private val perpetualService: GemPerpetualService,
    private val sessionRepository: SessionRepository,
) : GetPortfolioData {

    override suspend fun getPortfolioData(
        type: PortfolioType,
        period: ChartPeriod,
        currency: Currency,
    ): PortfolioData = when (type) {
        PortfolioType.Wallet -> getWalletData(period, currency)
        PortfolioType.Perpetuals -> getPerpetualData(period)
    }

    private suspend fun getWalletData(period: ChartPeriod, currency: Currency): PortfolioData {
        val walletId = checkNotNull(sessionRepository.getCurrentWallet()?.id) { "Missing current wallet" }
        val portfolio = portfolioService.syncWalletValues(walletId.id, period.toJson(), currency.toJson())
        val values = portfolio.values.map { it.decodeJson<ChartDateValue>() }
        val statistics = listOfNotNull(
            portfolio.allTimeHigh?.let { PortfolioStatistic.AllTimeHigh(it.decodeJson<ChartValuePercentage>()) },
            portfolio.allTimeLow?.let { PortfolioStatistic.AllTimeLow(it.decodeJson<ChartValuePercentage>()) },
        )
        return PortfolioData(
            charts = listOf(PortfolioChartData(chartType = PortfolioChartType.Value, values = values)),
            statistics = statistics,
            availablePeriods = walletChartPeriods,
        )
    }

    private suspend fun getPerpetualData(period: ChartPeriod): PortfolioData {
        val address = checkNotNull(sessionRepository.session().value?.wallet?.hyperliquidAccount?.address) {
            "Perpetual account is not available"
        }
        val portfolio = perpetualService.getPortfolio(Chain.HyperCore.string, address).decodeJson<PerpetualPortfolio>()
        val timeframe = portfolio.timeframeData(period)

        val charts = listOf(
            PortfolioChartData(chartType = PortfolioChartType.Pnl, values = timeframe?.pnlHistory.orEmpty()),
            PortfolioChartData(
                chartType = PortfolioChartType.Value,
                values = timeframe?.accountValueHistory.orEmpty().dropWhile { it.value == 0.0 },
            ),
        )
        val statistics = buildList {
            portfolio.accountSummary?.let { summary ->
                add(PortfolioStatistic.UnrealizedPnl(summary.unrealizedPnl))
                add(PortfolioStatistic.AccountLeverage(summary.accountLeverage))
                add(PortfolioStatistic.MarginUsage(PortfolioMarginUsage(accountValue = summary.accountValue, usage = summary.marginUsage)))
            }
            portfolio.allTime?.let { allTime ->
                allTime.pnlHistory.lastOrNull()?.let { add(PortfolioStatistic.AllTimePnl(it.value)) }
                add(PortfolioStatistic.Volume(allTime.volume))
            }
        }
        return PortfolioData(charts = charts, statistics = statistics, availablePeriods = portfolio.availablePeriods())
    }
}

private fun PerpetualPortfolio.availablePeriods(): List<ChartPeriod> = listOfNotNull(
    day?.let { ChartPeriod.Day },
    week?.let { ChartPeriod.Week },
    month?.let { ChartPeriod.Month },
    allTime?.let { ChartPeriod.Year },
    allTime?.let { ChartPeriod.All },
)

private fun PerpetualPortfolio.timeframeData(period: ChartPeriod): PerpetualPortfolioTimeframeData? = when (period) {
    ChartPeriod.Hour, ChartPeriod.Day -> day
    ChartPeriod.Week -> week
    ChartPeriod.Month -> month
    ChartPeriod.Year, ChartPeriod.All -> allTime
}
