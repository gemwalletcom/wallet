package com.gemwallet.android.ui.models

import com.gemwallet.android.model.ValueFormatter
import com.wallet.core.primitives.Asset
import java.math.BigDecimal

interface CryptoFormattedUIModel {
    val asset: Asset

    val cryptoAmount: Double

    val isZeroAmount: Boolean
        get() = cryptoAmount == 0.0

    val cryptoFormatted: String
        get() = ValueFormatter(style = ValueFormatter.Style.Short)
            .string(BigDecimal.valueOf(cryptoAmount), asset.symbol)
}
