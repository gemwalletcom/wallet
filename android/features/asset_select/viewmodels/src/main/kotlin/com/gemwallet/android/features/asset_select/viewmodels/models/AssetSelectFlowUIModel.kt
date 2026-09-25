package com.gemwallet.android.features.asset_select.viewmodels.models

import android.content.Context
import com.gemwallet.android.ui.localization.stringRes
import uniffi.gemstone.GemSelectAssetFlow

data class AssetSelectFlowUIModel(val title: String, val showsBalanceFilter: Boolean)

internal fun GemSelectAssetFlow.uiModel(context: Context): AssetSelectFlowUIModel = AssetSelectFlowUIModel(
    title = context.getString(title.stringRes()),
    showsBalanceFilter = balanceFilter,
)
