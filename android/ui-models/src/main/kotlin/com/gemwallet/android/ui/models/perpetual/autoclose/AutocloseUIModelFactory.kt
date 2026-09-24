package com.gemwallet.android.ui.models.perpetual.autoclose

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.text
import com.wallet.core.primitives.PerpetualPositionData
import com.wallet.core.primitives.TpslType
import uniffi.gemstone.GemAutocloseEstimator
import uniffi.gemstone.GemAutocloseField
import uniffi.gemstone.GemAutocloseViewState
import uniffi.gemstone.GemValueTone
import uniffi.gemstone.autocloseFieldState

object AutocloseUIModelFactory {

    fun create(position: PerpetualPositionData, takeProfit: GemAutocloseField, stopLoss: GemAutocloseField, state: GemAutocloseViewState): AutocloseUIModel {
        val estimator = GemAutocloseEstimator(
            entryPrice = position.position.entryPrice,
            positionSize = position.position.size,
            direction = position.position.direction.toGem(),
            leverage = position.position.leverage,
        )
        return AutocloseUIModel(
            positionRow = state.positionRow,
            priceRows = state.priceRows,
            takeProfit = createField(takeProfit, estimator, state.showsErrors),
            stopLoss = createField(stopLoss, estimator, state.showsErrors),
            confirmEnabled = state.confirmEnabled,
        )
    }

    fun createField(field: GemAutocloseField, estimator: GemAutocloseEstimator, showErrors: Boolean = true): AutocloseUIModel.Field {
        val state = autocloseFieldState(field, estimator, showErrors)
        return AutocloseUIModel.Field(
            type = state.tpslType.toPrimitives(),
            isProfit = state.isProfit,
            pnl = state.estimate,
            pnlDirection = state.tone,
            percentSuggestions = state.suggestions,
            validation = state.validation,
        )
    }
}
