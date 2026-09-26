package com.gemwallet.android.features.transfer.viewmodels.confirm.models

import android.content.Context
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.infoSheet
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.ListItemTagType
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.localization.title
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemConfirmFeeRow
import uniffi.gemstone.GemConfirmFeeValue
import uniffi.gemstone.GemFeeAmount
import uniffi.gemstone.GemInfoTopic

fun GemConfirmFeeRow.listItem(context: Context): ListItemModel {
    val title = title.text(context)
    val info = info.infoSheet()
    return when (val value = value) {
        GemConfirmFeeValue.Loading -> ListItemModel(title = title, subtitleTagType = ListItemTagType.Progress, info = info)

        is GemConfirmFeeValue.Unavailable -> ListItemModel(title = title, subtitle = value.text, info = info)

        is GemConfirmFeeValue.Ready -> ListItemModel(
            title = title,
            subtitle = value.text.value.text(),
            subtitleExtra = value.text.extra?.string(context),
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
