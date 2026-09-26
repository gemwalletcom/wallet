package com.gemwallet.android.features.assets.viewmodels.asset.models

import androidx.annotation.StringRes
import com.gemwallet.android.ui.components.list_item.ListItemSymbol
import com.gemwallet.android.ui.localization.toastRes
import com.gemwallet.android.ui.style.symbol
import uniffi.gemstone.GemPriceAlertToggle

data class PriceAlertMenuUIModel(@param:StringRes val toastRes: Int, val symbol: ListItemSymbol)

internal fun GemPriceAlertToggle.menu(): PriceAlertMenuUIModel = PriceAlertMenuUIModel(
    toastRes = toastRes(),
    symbol = symbol(),
)
