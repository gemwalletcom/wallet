package com.gemwallet.android.features.confirm.viewmodels.models

import android.content.Context
import com.gemwallet.android.domains.confirm.FeeUIModel
import com.gemwallet.android.ext.networkName
import com.gemwallet.android.ext.toChain
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.confirm.viewmodels.localization.title
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.ListItemTagType
import com.gemwallet.android.ui.localization.stringRes
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.BlockExplorerLink
import com.wallet.core.primitives.Chain
import uniffi.gemstone.AddressName
import uniffi.gemstone.AddressType
import uniffi.gemstone.GemConfirmDestination
import uniffi.gemstone.GemConfirmRowContent
import uniffi.gemstone.GemListRow
import uniffi.gemstone.contactInitials

sealed interface ConfirmRowUIModel {
    data class Row(val row: GemListRow) : ConfirmRowUIModel
    data class Item(val model: ListItemModel) : ConfirmRowUIModel
    data class Address(val title: String, val name: String?, val address: String, val chain: Chain, val explorerLink: BlockExplorerLink, val avatar: ListItemImage?) : ConfirmRowUIModel
    data class Validator(val title: String, val name: String, val address: String, val chain: Chain, val explorerLink: BlockExplorerLink) : ConfirmRowUIModel
    data class PaymentAsset(val model: ListItemModel, val selectable: Boolean) : ConfirmRowUIModel
}

internal fun GemConfirmRowContent.uiModel(context: Context): ConfirmRowUIModel? = when (this) {
    is GemConfirmRowContent.Row -> ConfirmRowUIModel.Row(row)
    is GemConfirmRowContent.Recipient -> uiModel(context)
    is GemConfirmRowContent.PaymentAsset -> ConfirmRowUIModel.PaymentAsset(model = ListItemModel(title = context.getString(R.string.transfer_pay_with), subtitle = symbol), selectable = selectable)
    is GemConfirmRowContent.Details -> null
}

private fun GemConfirmRowContent.Recipient.uiModel(context: Context): ConfirmRowUIModel {
    val title = context.getString(destination.title())
    return when (val destination = destination) {
        is GemConfirmDestination.Recipient -> ConfirmRowUIModel.Address(
            title = title,
            name = destination.name,
            address = destination.address,
            chain = chain.toChain(),
            explorerLink = link.toPrimitives(),
            avatar = addressName?.avatar(destination.name),
        )

        is GemConfirmDestination.Contract -> ConfirmRowUIModel.Address(
            title = title,
            name = null,
            address = destination.address,
            chain = chain.toChain(),
            explorerLink = link.toPrimitives(),
            avatar = null,
        )

        is GemConfirmDestination.Validator -> ConfirmRowUIModel.Validator(
            title = title,
            name = destination.name,
            address = destination.address,
            chain = chain.toChain(),
            explorerLink = link.toPrimitives(),
        )

        is GemConfirmDestination.Resource -> ConfirmRowUIModel.Item(ListItemModel(title = title, subtitle = context.getString(destination.resource.toPrimitives().stringRes())))

        is GemConfirmDestination.Provider -> ConfirmRowUIModel.Item(ListItemModel(title = title, subtitle = destination.name))
    }
}

private fun AddressName.avatar(name: String?): ListItemImage? {
    if (addressType != AddressType.CONTACT) return null
    val initials = name?.let { contactInitials(it) }
    return imageUrl?.takeIf { it.isNotEmpty() }?.let { ListItemImage.Stored(it, initials) } ?: initials?.let { ListItemImage.Initials(it) }
}

fun FeeUIModel.listItem(context: Context, feeAsset: Asset?, showsFeeAssetSymbol: Boolean = false): ListItemModel {
    val title = context.getString(R.string.transfer_network_fee)
    val info = InfoSheetEntity.NetworkFeeInfo(feeAsset?.id?.chain?.networkName().orEmpty(), feeAsset?.symbol.orEmpty())
    return when (this) {
        FeeUIModel.Calculating -> ListItemModel(title = title, subtitleTagType = ListItemTagType.Progress, info = info)

        is FeeUIModel.Unavailable -> ListItemModel(title = title, subtitle = text, info = info)

        is FeeUIModel.FeeInfo -> ListItemModel(
            title = title,
            subtitle = fiatAmount.ifEmpty { cryptoAmount },
            subtitleExtra = feeAsset?.symbol?.takeIf { showsFeeAssetSymbol && fiatAmount.isNotEmpty() },
            info = info,
        )
    }
}

internal fun verificationListItem(context: Context): ListItemModel = ListItemModel(
    title = context.getString(R.string.info_payment_verification_title),
    subtitleTagType = ListItemTagType.Pending,
    info = InfoSheetEntity.PaymentVerificationInfo,
)

internal fun FeeUIModel.FeeInfo.feeItems(context: Context): List<ListItemModel> = feeItems.map { (option, info) ->
    ListItemModel(title = context.getString(option.stringRes()), subtitle = info.cryptoAmount)
}
