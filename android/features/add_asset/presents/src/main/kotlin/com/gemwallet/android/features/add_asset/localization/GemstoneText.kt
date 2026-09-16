package com.gemwallet.android.features.add_asset.localization

import androidx.annotation.StringRes
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemAssetInfoKind

@StringRes
internal fun GemAssetInfoKind.stringRes(): Int = when (this) {
    GemAssetInfoKind.NAME -> R.string.asset_name
    GemAssetInfoKind.SYMBOL -> R.string.asset_symbol
    GemAssetInfoKind.DECIMALS -> R.string.asset_decimals
    GemAssetInfoKind.KIND -> R.string.common_type
}
