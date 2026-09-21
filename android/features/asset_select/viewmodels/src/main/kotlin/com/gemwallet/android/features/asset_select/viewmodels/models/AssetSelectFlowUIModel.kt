package com.gemwallet.android.features.asset_select.viewmodels.models

import android.content.Context
import com.gemwallet.android.features.asset_select.viewmodels.localization.stringRes
import uniffi.gemstone.GemAssetSubtitleStyle
import uniffi.gemstone.GemAssetTrailingStyle
import uniffi.gemstone.GemSelectAssetFlow

enum class AssetRowSubtitle {
    Network,
    Price,
}

enum class AssetRowTrailing {
    Balance,
    Toggle,
    Copy,
    None,
}

data class AssetSelectFlowUIModel(val title: String, val subtitle: AssetRowSubtitle, val trailing: AssetRowTrailing, val showsBalanceFilter: Boolean)

internal fun GemSelectAssetFlow.uiModel(context: Context): AssetSelectFlowUIModel = AssetSelectFlowUIModel(
    title = context.getString(title.stringRes()),
    showsBalanceFilter = balanceFilter,
    subtitle = when (rowStyle.subtitle) {
        GemAssetSubtitleStyle.NETWORK -> AssetRowSubtitle.Network
        GemAssetSubtitleStyle.PRICE -> AssetRowSubtitle.Price
    },
    trailing = when (rowStyle.trailing) {
        GemAssetTrailingStyle.BALANCE -> AssetRowTrailing.Balance
        GemAssetTrailingStyle.TOGGLE -> AssetRowTrailing.Toggle
        GemAssetTrailingStyle.COPY -> AssetRowTrailing.Copy
        GemAssetTrailingStyle.NONE -> AssetRowTrailing.None
    },
)
