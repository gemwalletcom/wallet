package com.gemwallet.android.features.assets.viewmodels.asset.models

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetData
import uniffi.gemstone.GemAssetDetails
import uniffi.gemstone.GemPriceAlertToggle

class AssetUIState(val assetInfo: AssetData, val details: GemAssetDetails, val priceAlertMenu: PriceAlertMenuUIModel = GemPriceAlertToggle.DISABLED.menu(), val sections: List<AssetDetailSectionUIModel> = emptyList()) {

    val asset: Asset get() = assetInfo.asset
}
