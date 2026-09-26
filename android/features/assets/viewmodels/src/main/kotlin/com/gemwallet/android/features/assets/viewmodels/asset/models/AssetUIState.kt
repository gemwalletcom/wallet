package com.gemwallet.android.features.assets.viewmodels.asset.models

import com.gemwallet.android.ui.components.banner.BannerRowUIModel
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetData
import uniffi.gemstone.GemAssetDetails
import uniffi.gemstone.GemPriceAlertToggle

class AssetUIState(
    val assetInfo: AssetData,
    val details: GemAssetDetails,
    val priceAlertMenu: PriceAlertMenuUIModel = GemPriceAlertToggle.DISABLED.menu(),
    val emptyTransactions: EmptyTransactionsUIModel = EmptyTransactionsUIModel(showsBuy = false, showsSwap = false),
    val banner: BannerRowUIModel?,
    val sections: List<AssetDetailSectionUIModel> = emptyList(),
) {

    val asset: Asset get() = assetInfo.asset
}
