package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.application.assets.cases.GetPortfolioData
import com.gemwallet.android.application.session.cases.GetCurrentWallet
import com.gemwallet.android.ext.hyperliquidAccount
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PortfolioData
import com.wallet.core.primitives.PortfolioType
import uniffi.gemstone.GemPortfolioDataInput
import uniffi.gemstone.GemPortfolioService
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

class GetPortfolioDataImpl(
    private val portfolioService: GemPortfolioService,
    private val getCurrentWallet: GetCurrentWallet,
) : GetPortfolioData {

    override suspend fun getPortfolioData(
        type: PortfolioType,
        period: ChartPeriod,
        currency: Currency,
    ): PortfolioData {
        val input = input(type, period, currency)
        return withContext(Dispatchers.IO) { portfolioService.portfolioData(input) }.decodeJson()
    }

    private suspend fun input(type: PortfolioType, period: ChartPeriod, currency: Currency): GemPortfolioDataInput = when (type) {
        PortfolioType.Wallet -> {
            val walletId = checkNotNull(getCurrentWallet.getCurrentWallet()?.id) { "Missing current wallet" }
            GemPortfolioDataInput.Wallet(walletId.id, period.toJson(), currency.toGem())
        }
        PortfolioType.Perpetuals -> {
            val address = checkNotNull(getCurrentWallet.getCurrentWallet()?.hyperliquidAccount?.address) {
                "Perpetual account is not available"
            }
            GemPortfolioDataInput.Perpetuals(Chain.HyperCore.string, address, period.toJson())
        }
    }
}
