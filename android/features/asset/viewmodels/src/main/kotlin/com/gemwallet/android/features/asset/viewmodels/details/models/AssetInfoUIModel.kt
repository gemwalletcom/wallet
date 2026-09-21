package com.gemwallet.android.features.asset.viewmodels.details.models

import androidx.annotation.StringRes
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.ui.components.banner.BannerRowUIModel
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.VerificationStatus
import uniffi.gemstone.GemAssetDetailsState
import uniffi.gemstone.GemAssetNetworkDestination
import uniffi.gemstone.GemBalanceRow
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemPriceAlertToggle
import uniffi.gemstone.GemValueTone

class AssetInfoUIModel(
    val assetInfo: AssetInfo,
    val name: String = "",
    val iconUrl: Any? = null,
    val priceValue: String = "0",
    val priceDayChanges: String = "0",
    val priceChangedType: GemValueTone = GemValueTone.POSITIVE,
    val tokenType: AssetType = AssetType.NATIVE,
    val accountInfoUIModel: AccountInfoUIModel = AccountInfoUIModel(),
    val isBuyEnabled: Boolean = false,
    val isSwapEnabled: Boolean = false,
    val swapPayAssetId: AssetId? = null,
    val swapReceiveAssetId: AssetId? = null,
    val explorerName: String = "",
    val explorerAddressUrl: String? = null,
    val explorerTokenUrl: String? = null,
    val verificationStatus: VerificationStatus? = null,
    val networkDestination: GemAssetNetworkDestination? = null,
    val shareUrl: String = "",
    val detailsState: GemAssetDetailsState,
    val priceAlertMenu: PriceAlertMenuUIModel = GemPriceAlertToggle.DISABLED.menu(),
    val emptyTransactions: EmptyTransactionsUIModel = EmptyTransactionsUIModel(showsBuy = false, showsSwap = false),
    val banners: List<BannerRowUIModel>,
    val priceListItem: ListItemModel = ListItemModel(title = ""),
    val sections: List<SectionUIModel> = emptyList(),
) {

    val asset: Asset get() = assetInfo.asset

    data class AccountInfoUIModel(val totalBalance: String = "0", val totalFiat: String = "", val owner: String = "")

    data class SectionUIModel(@StringRes val title: Int?, val rows: List<RowUIModel>)

    sealed interface RowUIModel {
        data object Price : RowUIModel
        data class Network(val name: String) : RowUIModel
        data class Balance(val type: BalanceViewType, val model: ListItemModel, val url: String? = null) : RowUIModel
        data class Earn(val model: ListItemModel) : RowUIModel
        data class Row(val row: GemListRow) : RowUIModel
    }

    enum class BalanceViewType {
        Available,
        Stake,
        Earn,
        PendingUnconfirmed,
        Reserved,
    }
}

internal fun GemBalanceRow.viewType(): AssetInfoUIModel.BalanceViewType = when (this) {
    is GemBalanceRow.Available -> AssetInfoUIModel.BalanceViewType.Available
    is GemBalanceRow.Staked -> AssetInfoUIModel.BalanceViewType.Stake
    is GemBalanceRow.Earn -> AssetInfoUIModel.BalanceViewType.Earn
    is GemBalanceRow.PendingUnconfirmed -> AssetInfoUIModel.BalanceViewType.PendingUnconfirmed
    is GemBalanceRow.Reserved -> AssetInfoUIModel.BalanceViewType.Reserved
}
