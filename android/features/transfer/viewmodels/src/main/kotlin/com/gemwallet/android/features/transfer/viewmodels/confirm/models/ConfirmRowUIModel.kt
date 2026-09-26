package com.gemwallet.android.features.transfer.viewmodels.confirm.models

import android.content.Context
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.infoSheet
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.ListItemTagType
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.localization.title
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemAddressRow
import uniffi.gemstone.GemConfirmFeeRow
import uniffi.gemstone.GemConfirmRowContent
import uniffi.gemstone.GemFeeAmount
import uniffi.gemstone.GemInfoTopic
import uniffi.gemstone.GemListRow

sealed interface ConfirmRowUIModel {
    data class Row(val row: GemListRow) : ConfirmRowUIModel
    data class Item(val model: ListItemModel) : ConfirmRowUIModel
    data class Address(val row: GemAddressRow) : ConfirmRowUIModel
    data class PaymentAsset(val model: ListItemModel, val selectable: Boolean, val assetIds: List<AssetId>) : ConfirmRowUIModel
}

internal fun GemConfirmRowContent.uiModel(context: Context): ConfirmRowUIModel? = when (this) {
    is GemConfirmRowContent.Row -> ConfirmRowUIModel.Row(row)

    is GemConfirmRowContent.Recipient -> ConfirmRowUIModel.Address(row)

    is GemConfirmRowContent.PaymentAsset -> ConfirmRowUIModel.PaymentAsset(
        model = ListItemModel(title = context.getString(R.string.transfer_pay_with), subtitle = symbol),
        selectable = selectable,
        assetIds = assetIds.mapNotNull { it.toAssetId() },
    )

    is GemConfirmRowContent.Details -> null
}

fun GemConfirmFeeRow.listItem(context: Context, feeAsset: Asset?): ListItemModel {
    val title = context.getString(R.string.transfer_network_fee)
    val info = networkFeeInfo(feeAsset)
    return when (this) {
        GemConfirmFeeRow.Loading -> ListItemModel(title = title, subtitleTagType = ListItemTagType.Progress, info = info)

        is GemConfirmFeeRow.Unavailable -> ListItemModel(title = title, subtitle = text, info = info)

        is GemConfirmFeeRow.Ready -> ListItemModel(
            title = title,
            subtitle = text.value.text(),
            subtitleExtra = text.extra?.string(context),
            info = info,
        )
    }
}

fun GemFeeAmount?.networkFeeListItem(context: Context, feeAsset: Asset): ListItemModel = ListItemModel(
    title = context.getString(R.string.transfer_network_fee),
    subtitle = this?.let { fee -> fee.fiat?.text() ?: fee.amount.text() },
    info = networkFeeInfo(feeAsset),
)

private fun networkFeeInfo(feeAsset: Asset?) = feeAsset?.let { GemInfoTopic.NetworkFee(it.toGem()).infoSheet() }

internal fun verificationListItem(context: Context): ListItemModel = ListItemModel(
    title = context.getString(R.string.info_payment_verification_title),
    subtitleTagType = ListItemTagType.Pending,
    info = GemInfoTopic.PaymentVerification.infoSheet(),
)
