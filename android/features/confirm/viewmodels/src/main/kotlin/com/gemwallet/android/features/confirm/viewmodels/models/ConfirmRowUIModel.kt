package com.gemwallet.android.features.confirm.viewmodels.models

import android.content.Context
import com.gemwallet.android.domains.confirm.FeeUIModel
import com.gemwallet.android.ext.networkName
import com.gemwallet.android.ext.toAssetId
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
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.BlockExplorerLink
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemAvatar
import uniffi.gemstone.GemConfirmDestination
import uniffi.gemstone.GemConfirmRowContent
import uniffi.gemstone.GemListRow

sealed interface ConfirmRowUIModel {
    data class Row(val row: GemListRow) : ConfirmRowUIModel
    data class Item(val model: ListItemModel) : ConfirmRowUIModel
    data class Address(val title: String, val text: String, val address: String, val chain: Chain, val explorerLink: BlockExplorerLink, val avatar: ListItemImage?) : ConfirmRowUIModel
    data class Validator(val title: String, val name: String, val address: String, val chain: Chain, val explorerLink: BlockExplorerLink) : ConfirmRowUIModel
    data class PaymentAsset(val model: ListItemModel, val selectable: Boolean, val assetIds: List<AssetId>) : ConfirmRowUIModel
}

internal fun GemConfirmRowContent.uiModel(context: Context): ConfirmRowUIModel? = when (this) {
    is GemConfirmRowContent.Row -> ConfirmRowUIModel.Row(row)

    is GemConfirmRowContent.Recipient -> uiModel(context)

    is GemConfirmRowContent.PaymentAsset -> ConfirmRowUIModel.PaymentAsset(
        model = ListItemModel(title = context.getString(R.string.transfer_pay_with), subtitle = symbol),
        selectable = selectable,
        assetIds = assetIds.mapNotNull { it.toAssetId() },
    )

    is GemConfirmRowContent.Details -> null
}

private fun GemConfirmRowContent.Recipient.uiModel(context: Context): ConfirmRowUIModel {
    val title = context.getString(destination.title())
    return when (destination) {
        is GemConfirmDestination.Resource -> ConfirmRowUIModel.Item(
            ListItemModel(title = title, subtitle = context.getString((destination as GemConfirmDestination.Resource).resource.toPrimitives().stringRes())),
        )

        is GemConfirmDestination.Validator -> ConfirmRowUIModel.Validator(
            title = title,
            name = name.orEmpty(),
            address = address,
            chain = chain.toChain(),
            explorerLink = link.toPrimitives(),
        )

        else -> ConfirmRowUIModel.Address(
            title = title,
            text = text,
            address = address,
            chain = chain.toChain(),
            explorerLink = link.toPrimitives(),
            avatar = avatar?.listItemImage(),
        )
    }
}

private fun GemAvatar.listItemImage(): ListItemImage = imageUrl?.let { ListItemImage.Stored(it, initials) } ?: ListItemImage.Initials(initials)

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
