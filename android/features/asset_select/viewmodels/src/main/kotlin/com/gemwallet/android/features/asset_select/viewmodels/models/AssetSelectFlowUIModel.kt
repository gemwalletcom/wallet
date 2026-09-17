package com.gemwallet.android.features.asset_select.viewmodels.models

import android.content.Context
import com.gemwallet.android.features.asset_select.viewmodels.localization.stringRes
import uniffi.gemstone.GemAssetSubtitleStyle
import uniffi.gemstone.GemAssetTrailingStyle
import uniffi.gemstone.GemSelectAssetFlow

data class AssetSelectFlowUIModel(
    val title: String,
    val showsSymbol: Boolean,
    val subtitle: GemAssetSubtitleStyle,
    val trailing: GemAssetTrailingStyle,
    val showsBalanceFilter: Boolean,
)

internal fun GemSelectAssetFlow.uiModel(context: Context): AssetSelectFlowUIModel = AssetSelectFlowUIModel(
    title = context.getString(title.stringRes()),
    showsSymbol = rowStyle.showsSymbol,
    showsBalanceFilter = balanceFilter,
    subtitle = rowStyle.subtitle,
    trailing = rowStyle.trailing,
)
