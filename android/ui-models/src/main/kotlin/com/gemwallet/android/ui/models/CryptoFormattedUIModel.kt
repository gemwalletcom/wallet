package com.gemwallet.android.ui.models

import com.gemwallet.android.model.ValueFormatter
import java.math.BigDecimal

interface CryptoFormattedUIModel : CryptoAmountUIModel, AssetUIModel {
    val cryptoFormatted: String
        get() = ValueFormatter(style = ValueFormatter.Style.Short)
            .string(BigDecimal.valueOf(cryptoAmount), asset.symbol)
}