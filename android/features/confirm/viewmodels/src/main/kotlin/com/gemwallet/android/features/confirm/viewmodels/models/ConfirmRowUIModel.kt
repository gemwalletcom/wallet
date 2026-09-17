package com.gemwallet.android.features.confirm.viewmodels.models

import android.content.Context
import com.gemwallet.android.domains.confirm.ConfirmProperty
import com.gemwallet.android.domains.confirm.FeeUIModel
import com.gemwallet.android.features.confirm.viewmodels.localization.titleRes
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoSheetEntity
import uniffi.gemstone.contactInitials
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.ListItemTagType
import com.gemwallet.android.ui.components.list_item.listItemImage
import com.gemwallet.android.ui.localization.stringRes
import com.wallet.core.primitives.AddressType
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.BlockExplorerLink
import com.wallet.core.primitives.Chain

sealed interface ConfirmRowUIModel {
    data class Item(val model: ListItemModel) : ConfirmRowUIModel
    data class Address(
        val title: String,
        val name: String?,
        val address: String,
        val chain: Chain,
        val explorerLink: BlockExplorerLink?,
        val avatar: ListItemImage?,
    ) : ConfirmRowUIModel
    data class Validator(
        val title: String,
        val name: String,
        val address: String,
        val explorerLink: BlockExplorerLink,
    ) : ConfirmRowUIModel
    data class Network(val chain: Chain, val name: String) : ConfirmRowUIModel
}

internal fun ConfirmProperty.uiModel(context: Context): ConfirmRowUIModel = when (this) {
    is ConfirmProperty.Memo -> ConfirmRowUIModel.Item(ListItemModel(title = context.getString(R.string.transfer_memo), subtitle = data))
    is ConfirmProperty.Destination.Provider -> ConfirmRowUIModel.Item(ListItemModel(title = context.getString(titleRes()), subtitle = data))
    is ConfirmProperty.Destination.Generic -> ConfirmRowUIModel.Item(ListItemModel(title = context.getString(titleRes()), subtitle = appName))
    is ConfirmProperty.Destination.Resource -> ConfirmRowUIModel.Item(ListItemModel(title = context.getString(titleRes()), subtitle = context.getString(resource.stringRes())))
    is ConfirmProperty.Destination.Stake -> {
        val address = address
        val explorerLink = explorerLink
        if (address != null && explorerLink != null) {
            ConfirmRowUIModel.Validator(title = context.getString(titleRes()), name = data, address = address, explorerLink = explorerLink)
        } else {
            ConfirmRowUIModel.Item(ListItemModel(title = context.getString(titleRes()), subtitle = data))
        }
    }
    is ConfirmProperty.Destination.Transfer -> ConfirmRowUIModel.Address(
        title = context.getString(titleRes()),
        name = domain,
        address = address,
        chain = chain,
        explorerLink = explorerLink,
        avatar = avatar(),
    )
    is ConfirmProperty.Destination.Contract -> ConfirmRowUIModel.Address(
        title = context.getString(titleRes()),
        name = null,
        address = address,
        chain = chain,
        explorerLink = explorerLink,
        avatar = null,
    )
    is ConfirmProperty.Source -> ConfirmRowUIModel.Item(
        ListItemModel(title = context.getString(R.string.common_wallet), subtitle = walletRow.name, image = walletRow.listItemImage()),
    )
    is ConfirmProperty.Network -> ConfirmRowUIModel.Network(chain = chain, name = name)
}

private fun ConfirmProperty.Destination.Transfer.avatar(): ListItemImage? {
    if (addressType != AddressType.Contact) return null
    val initials = domain?.let { contactInitials(it) }
    return imageUrl?.takeIf { it.isNotEmpty() }?.let { ListItemImage.Stored(it, initials) } ?: initials?.let { ListItemImage.Initials(it) }
}

fun FeeUIModel.listItem(context: Context, feeAsset: Asset?): ListItemModel {
    val title = context.getString(R.string.transfer_network_fee)
    val info = InfoSheetEntity.NetworkFeeInfo(feeAsset?.name.orEmpty(), feeAsset?.symbol.orEmpty())
    return when (this) {
        FeeUIModel.Calculating -> ListItemModel(title = title, subtitleTagType = ListItemTagType.Progress, info = info)
        FeeUIModel.Error -> ListItemModel(title = title, subtitle = "~", info = info)
        is FeeUIModel.FeeInfo -> ListItemModel(title = title, subtitle = cryptoAmount, subtitleExtra = fiatAmount.takeIf { it.isNotEmpty() }, info = info)
    }
}

internal fun FeeUIModel.FeeInfo.feeItems(context: Context): List<ListItemModel> = feeItems.map { (option, info) ->
    ListItemModel(title = context.getString(option.stringRes()), subtitle = info.cryptoAmount)
}
