package com.gemwallet.android.domains.perpetual.autoclose

import com.wallet.core.primitives.CancelOrderData
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualModifyPositionType
import com.wallet.core.primitives.TPSLOrderData

class AutocloseModifyBuilder(private val direction: PerpetualDirection) {

    fun canBuild(takeProfit: AutocloseField, stopLoss: AutocloseField): Boolean {
        val takeProfitOk = takeProfit.price == null || takeProfit.isValid
        val stopLossOk = stopLoss.price == null || stopLoss.isValid
        if (!takeProfitOk || !stopLossOk) return false
        return takeProfit.shouldUpdate || stopLoss.shouldUpdate
    }

    fun build(
        assetIndex: Int,
        takeProfit: AutocloseField,
        stopLoss: AutocloseField,
    ): List<PerpetualModifyPositionType> {
        val result = mutableListOf<PerpetualModifyPositionType>()

        val cancels = listOf(takeProfit, stopLoss).mapNotNull { field ->
            if (!field.shouldCancel) return@mapNotNull null
            val orderId = field.orderId ?: return@mapNotNull null
            CancelOrderData(assetIndex = assetIndex, orderId = orderId.toLong())
        }
        if (cancels.isNotEmpty()) {
            result += PerpetualModifyPositionType.Cancel(cancels)
        }

        if (takeProfit.shouldSet || stopLoss.shouldSet) {
            result += PerpetualModifyPositionType.Tpsl(
                TPSLOrderData(
                    direction = direction,
                    takeProfit = takeProfit.formattedPrice?.takeIf { takeProfit.shouldSet },
                    stopLoss = stopLoss.formattedPrice?.takeIf { stopLoss.shouldSet },
                    size = "0",
                ),
            )
        }

        return result
    }
}
