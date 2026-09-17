package com.gemwallet.android.features.activities.viewmodels.models

import android.content.Context
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.duration.formatEstimatedConfirmation
import com.gemwallet.android.domains.transaction.values.TransactionDetailsValue
import com.gemwallet.android.features.activities.viewmodels.localization.stringRes
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.image.iconModel
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.ListItemTagType
import com.gemwallet.android.ui.localization.infoDescriptionRes
import com.gemwallet.android.ui.localization.statusLabelRes
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.style.badgeIconRes
import com.gemwallet.android.ui.style.textStyle
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.BlockExplorerLink
import com.wallet.core.primitives.Chain

sealed interface TransactionDetailsRowUIModel {
    data class Item(val model: ListItemModel, val url: String? = null) : TransactionDetailsRowUIModel
    data class Address(
        val title: String,
        val address: String,
        val chain: Chain?,
        val name: String?,
        val explorerLink: BlockExplorerLink?,
    ) : TransactionDetailsRowUIModel
    data class Fee(val model: ListItemModel) : TransactionDetailsRowUIModel
    data class SwapProgress(val model: SwapProgressUIModel) : TransactionDetailsRowUIModel
    data class Value(val value: TransactionDetailsValue) : TransactionDetailsRowUIModel
}

internal fun TransactionDetailsValue.uiModel(context: Context, asset: Asset): TransactionDetailsRowUIModel = when (this) {
    is TransactionDetailsValue.Date -> TransactionDetailsRowUIModel.Item(ListItemModel(title = context.getString(R.string.transaction_date), subtitle = data))
    is TransactionDetailsValue.Memo -> TransactionDetailsRowUIModel.Item(ListItemModel(title = context.getString(R.string.transfer_memo), subtitle = data))
    is TransactionDetailsValue.ResourceType -> TransactionDetailsRowUIModel.Item(ListItemModel(title = context.getString(R.string.stake_resource), subtitle = context.getString(data.stringRes())))
    is TransactionDetailsValue.Pnl -> TransactionDetailsRowUIModel.Item(ListItemModel(title = context.getString(R.string.perpetual_pnl), subtitle = value, subtitleStyle = direction.textStyle()))
    is TransactionDetailsValue.Price -> TransactionDetailsRowUIModel.Item(ListItemModel(title = context.getString(R.string.asset_price), subtitle = data))
    is TransactionDetailsValue.EstimatedConfirmation -> TransactionDetailsRowUIModel.Item(
        ListItemModel(
            title = context.getString(R.string.transaction_estimated_confirmation),
            subtitle = formatEstimatedConfirmation(seconds),
            info = InfoSheetEntity.EstimatedConfirmationInfo(asset.chain),
        ),
    )
    is TransactionDetailsValue.Explorer -> TransactionDetailsRowUIModel.Item(ListItemModel(title = context.getString(R.string.transaction_view_on, name)), url = url)
    is TransactionDetailsValue.Destination.Provider -> TransactionDetailsRowUIModel.Item(ListItemModel(title = context.getString(stringRes()), subtitle = data))
    is TransactionDetailsValue.Destination -> TransactionDetailsRowUIModel.Address(
        title = context.getString(stringRes()),
        address = data,
        chain = chain,
        name = name,
        explorerLink = explorerLink,
    )
    is TransactionDetailsValue.Status -> TransactionDetailsRowUIModel.Item(
        ListItemModel(
            title = context.getString(R.string.transaction_status),
            subtitle = context.getString(data.statusLabelRes()),
            subtitleStyle = status.tone.textStyle(),
            subtitleTagType = if (status.showsProgress) ListItemTagType.Progress else ListItemTagType.None,
            info = InfoSheetEntity.TransactionInfo(icon = asset.iconModel(), state = data, badgeIcon = status.tone.badgeIconRes(), description = status.tone.infoDescriptionRes()),
        ),
    )
    is TransactionDetailsValue.Fee -> TransactionDetailsRowUIModel.Fee(
        ListItemModel(
            title = context.getString(R.string.transfer_network_fee),
            subtitle = value,
            subtitleExtra = equivalent.takeIf { it.isNotEmpty() },
            info = InfoSheetEntity.NetworkFeeInfo(this.asset.name, this.asset.symbol),
        ),
    )
    is TransactionDetailsValue.SwapProgress -> TransactionDetailsRowUIModel.SwapProgress(uiModel(context))
    is TransactionDetailsValue.Amount,
    is TransactionDetailsValue.Network,
    is TransactionDetailsValue.Rate,
    is TransactionDetailsValue.SwapAgain -> TransactionDetailsRowUIModel.Value(this)
}
