package com.gemwallet.android.blockchain.services

import com.gemwallet.android.blockchain.gemstone.toDTO
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChartCandleStick
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.PerpetualData
import com.wallet.core.primitives.PerpetualPositionsSummary
import uniffi.gemstone.GemGateway

class PerpetualService(
    private val gateway: GemGateway,
) {
    suspend fun getPerpetualsData(chain: Chain = Chain.HyperCore): List<PerpetualData> {
        val response = try {
            gateway.getPerpetualsData(chain.string)
        } catch (_: Throwable) {
            return emptyList()
        }
        return response.mapNotNull { it.toDTO() }
    }

    suspend fun getPositions(chain: Chain = Chain.HyperCore, address: String): PerpetualPositionsSummary? {
        val response = try {
            gateway.getPositions(chain.string, address)
        } catch (_: Throwable) {
            return null
        }
        return response.toDTO()
    }

    suspend fun getCandleSticks(chain: Chain = Chain.HyperCore, symbol: String, period: ChartPeriod): List<ChartCandleStick> {
        val response = try {
            gateway.getPerpetualCandlesticks(chain.string, symbol, period.string)
        } catch (_: Throwable) {
            return emptyList()
        }
        return response.map { it.toDTO() }
    }
}
