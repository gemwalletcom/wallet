package com.gemwallet.android.features.assets.viewmodels.select.models

import android.content.Context
import com.gemwallet.android.ui.localization.stringRes
import uniffi.gemstone.GemSelectAssetFlow

data class SelectAssetFlowUIModel(val title: String, val showsBalanceFilter: Boolean)

internal fun GemSelectAssetFlow.uiModel(context: Context): SelectAssetFlowUIModel = SelectAssetFlowUIModel(
    title = context.getString(title.stringRes()),
    showsBalanceFilter = balanceFilter,
)
