package com.gemwallet.android.features.asset.viewmodels.details.models

import androidx.annotation.StringRes
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.ui.components.banner.BannerRowUIModel
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemAssetDetails
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemPriceAlertToggle

class AssetInfoUIModel(
    val assetInfo: AssetInfo,
    val details: GemAssetDetails,
    val priceAlertMenu: PriceAlertMenuUIModel = GemPriceAlertToggle.DISABLED.menu(),
    val emptyTransactions: EmptyTransactionsUIModel = EmptyTransactionsUIModel(showsBuy = false, showsSwap = false),
    val banners: List<BannerRowUIModel>,
    val sections: List<SectionUIModel> = emptyList(),
) {

    val asset: Asset get() = assetInfo.asset

    data class SectionUIModel(@StringRes val title: Int?, val rows: List<RowUIModel>)

    sealed interface RowUIModel {
        data class Balance(val model: ListItemModel, val action: AssetDetailsAction?) : RowUIModel
        data class Row(val row: GemListRow, val action: AssetDetailsAction?) : RowUIModel
    }
}
