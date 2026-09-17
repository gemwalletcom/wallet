package com.gemwallet.android.ui.components.perpetual

import android.content.Context
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemPerpetual
import uniffi.gemstone.PerpetualProvider

fun autocloseSummaryLines(context: Context, takeProfitText: String?, stopLossText: String?): List<String> =
    GemPerpetual(PerpetualProvider.HYPERCORE).use { perpetual ->
        listOfNotNull(
            takeProfitText?.let { perpetual.triggerOrderText(context.getString(R.string.perpetual_take_profit), it) },
            stopLossText?.let { perpetual.triggerOrderText(context.getString(R.string.perpetual_stop_loss), it) },
        )
    }
