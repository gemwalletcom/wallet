package com.gemwallet.android.features.asset_select.viewmodels.models

import android.content.Context
import com.gemwallet.android.features.asset_select.viewmodels.localization.stringRes
import uniffi.gemstone.GemAssetRowSubtitle
import uniffi.gemstone.GemAssetRowTrailing
import uniffi.gemstone.GemSelectAssetFlow

enum class AssetRowSubtitleStyle {
    Network,
    Price,
}

enum class AssetRowTrailingStyle {
    Balance,
    Toggle,
    Copy,
    None,
}

data class AssetSelectFlowUIModel(
    val title: String,
    val showsSymbol: Boolean,
    val subtitle: AssetRowSubtitleStyle,
    val trailing: AssetRowTrailingStyle,
    val showsBalanceFilter: Boolean,
)

internal fun GemSelectAssetFlow.uiModel(context: Context): AssetSelectFlowUIModel = AssetSelectFlowUIModel(
    title = context.getString(title.stringRes()),
    showsSymbol = row.showsSymbol,
    showsBalanceFilter = balanceFilter,
    subtitle = when (row.subtitle) {
        GemAssetRowSubtitle.NETWORK -> AssetRowSubtitleStyle.Network
        GemAssetRowSubtitle.PRICE -> AssetRowSubtitleStyle.Price
    },
    trailing = when (row.trailing) {
        GemAssetRowTrailing.BALANCE -> AssetRowTrailingStyle.Balance
        GemAssetRowTrailing.TOGGLE -> AssetRowTrailingStyle.Toggle
        GemAssetRowTrailing.COPY -> AssetRowTrailingStyle.Copy
        GemAssetRowTrailing.NONE -> AssetRowTrailingStyle.None
    },
)
