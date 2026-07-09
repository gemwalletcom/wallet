package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.GetPortfolioData
import com.gemwallet.android.application.assets.coordinators.walletChartPeriods
import com.gemwallet.android.blockchain.services.PerpetualService
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.gemwallet.android.ext.hyperliquidAccount
import com.gemwallet.android.ext.secondsToMillis
import com.gemwallet.android.model.getTotalAmount
import com.wallet.core.primitives.ChartDateValue
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualPortfolio
import com.wallet.core.primitives.PerpetualPortfolioTimeframeData
import com.wallet.core.primitives.PortfolioAsset
import com.wallet.core.primitives.PortfolioAssetsRequest
import com.wallet.core.primitives.PortfolioChartData
import com.wallet.core.primitives.PortfolioChartType
import com.wallet.core.primitives.PortfolioData
import com.wallet.core.primitives.PortfolioMarginUsage
import com.wallet.core.primitives.PortfolioStatistic
import com.wallet.core.primitives.PortfolioType
import kotlinx.coroutines.flow.firstOrNull
import java.math.BigInteger

class GetPortfolioDataImpl(
    private val gemDeviceApiClient: GemDeviceApiClient,
    private val assetsRepository: AssetsRepository,
    private val perpetualService: PerpetualService,
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
        val rate = checkNotNull(assetsRepository.getCurrencyRate(currency).firstOrNull()?.rate) {
            "Missing currency rate for ${currency.string}"
        }
        val assets = assetsRepository.getAssetsInfo().firstOrNull().orEmpty()
            .mapNotNull { assetInfo ->
                val total = assetInfo.balance.balance.getTotalAmount()
                if (total <= BigInteger.ZERO) return@mapNotNull null
                PortfolioAsset(assetId = assetInfo.asset.id, value = total.toString())
            }
        val portfolio = gemDeviceApiClient.getPortfolioAssets(
            period = period.string,
            request = PortfolioAssetsRequest(assets = assets),
        )
        val values = portfolio.values
            .sortedBy { it.timestamp }
            .map { ChartDateValue(date = it.timestamp.toLong().secondsToMillis(), value = it.value * rate) }
        val statistics = listOfNotNull(
            portfolio.allTimeHigh?.let { PortfolioStatistic.AllTimeHigh(it) },
            portfolio.allTimeLow?.let { PortfolioStatistic.AllTimeLow(it) },
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
        val portfolio = perpetualService.getPortfolio(address = address)
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
