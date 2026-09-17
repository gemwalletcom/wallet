package com.gemwallet.android.features.earn.delegation.models

import android.content.Context
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.features.earn.delegation.viewmodels.localization.stringRes
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.ListItemTextStyle
import com.gemwallet.android.ui.localization.stateRes
import com.gemwallet.android.ui.models.RewardsInfoUIModel
import com.gemwallet.android.ui.style.textStyle
import com.wallet.core.primitives.DelegationValidator
import uniffi.gemstone.GemDelegationAction
import uniffi.gemstone.GemDelegationRow
import uniffi.gemstone.GemDelegationStatus
import uniffi.gemstone.GemPercentageStyle

class DelegationProperties(
    val rows: List<DelegationRowUIModel>,
    val rewards: RewardsInfoUIModel,
)

sealed interface DelegationRowUIModel {
    data class Item(val model: ListItemModel, val url: String? = null) : DelegationRowUIModel
    data object Rewards : DelegationRowUIModel
}

data class DelegationActionUIModel(
    val action: GemDelegationAction,
    val model: ListItemModel,
)

internal fun GemDelegationRow.uiModel(
    context: Context,
    validator: DelegationValidator,
    validatorName: String,
    validatorUrl: String?,
    status: GemDelegationStatus,
    availableIn: String,
): DelegationRowUIModel? = when (this) {
    GemDelegationRow.APR -> DelegationRowUIModel.Item(
        ListItemModel(
            title = context.getString(R.string.stake_apr, ""),
            subtitle = validator.apr.formatAsPercentage(style = GemPercentageStyle.UNSIGNED),
            subtitleStyle = if (validator.isActive) ListItemTextStyle.Positive else ListItemTextStyle.Secondary,
        ),
    )
    GemDelegationRow.PROVIDER -> DelegationRowUIModel.Item(ListItemModel(title = context.getString(R.string.stake_validator), subtitle = validatorName), url = validatorUrl)
    GemDelegationRow.COMPLETION_DATE -> status.completion?.let {
        DelegationRowUIModel.Item(ListItemModel(title = context.getString(it.stringRes()), subtitle = availableIn))
    }
    GemDelegationRow.STATUS -> DelegationRowUIModel.Item(
        ListItemModel(title = context.getString(R.string.transaction_status), subtitle = context.getString(status.stateRes()), subtitleStyle = status.tone.textStyle()),
    )
    GemDelegationRow.REWARDS -> DelegationRowUIModel.Rewards
}

internal fun GemDelegationAction.uiModel(context: Context): DelegationActionUIModel = DelegationActionUIModel(
    action = this,
    model = ListItemModel(title = context.getString(stringRes())),
)
