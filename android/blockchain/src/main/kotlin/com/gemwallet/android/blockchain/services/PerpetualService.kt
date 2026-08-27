package com.gemwallet.android.blockchain.services

import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChartCandleStick
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.PerpetualAccountMode
import com.wallet.core.primitives.PerpetualPortfolio
import uniffi.gemstone.GemGateway

class PerpetualService(
    private val gateway: GemGateway,
) {

    suspend fun getCandleSticks(chain: Chain = Chain.HyperCore, symbol: String, period: ChartPeriod): List<ChartCandleStick> {
        return gateway.getPerpetualCandlesticks(chain.string, symbol, period.string).map { it.decodeJson() }
    }

    suspend fun getPortfolio(chain: Chain = Chain.HyperCore, address: String): PerpetualPortfolio =
        gateway.getPerpetualPortfolio(chain.string, address).decodeJson()
}
