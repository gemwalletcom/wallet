package com.gemwallet.android.features.confirm.models

import com.gemwallet.android.model.CurrencyFormatter
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualModifyConfirmData
import com.wallet.core.primitives.PerpetualModifyPositionType

object PerpetualModifyAutocloseFactory {

    private const val ClearedPlaceholder: String = "-"

    fun create(data: PerpetualModifyConfirmData): ConfirmDetailElement.PerpetualModifyAutoclose? {
        val tpsl = data.modifyTypes
            .filterIsInstance<PerpetualModifyPositionType.Tpsl>()
            .firstOrNull()?.content
        val cancelOrderIds = data.modifyTypes
            .filterIsInstance<PerpetualModifyPositionType.Cancel>()
            .flatMap { it.content }
            .map { it.orderId }
            .toSet()
        val takeProfitCanceled = data.takeProfitOrderId != null &&
            data.takeProfitOrderId in cancelOrderIds
        val stopLossCanceled = data.stopLossOrderId != null &&
            data.stopLossOrderId in cancelOrderIds
        val formatter = CurrencyFormatter(currency = Currency.USD)
        val takeProfitText: String? = when {
            tpsl?.takeProfit != null -> tpsl.takeProfit?.toDoubleOrNull()?.let(formatter::string)
            takeProfitCanceled -> ClearedPlaceholder
            else -> null
        }
        val stopLossText: String? = when {
            tpsl?.stopLoss != null -> tpsl.stopLoss?.toDoubleOrNull()?.let(formatter::string)
            stopLossCanceled -> ClearedPlaceholder
            else -> null
        }
        if (takeProfitText == null && stopLossText == null) return null
        return ConfirmDetailElement.PerpetualModifyAutoclose(takeProfitText, stopLossText)
    }
}
