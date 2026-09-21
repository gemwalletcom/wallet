package com.gemwallet.android.features.referral.viewmodels.models

import android.content.Context
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.localization.text
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemRewardsRedemption
import uniffi.gemstone.GemRewardsState

data class RewardRedemptionUIModel(val redemption: GemRewardsRedemption, val model: ListItemModel, val confirmationMessage: String)

internal fun GemRewardsState.infoRows(context: Context): List<ListItemModel> = infoRows.mapNotNull { row ->
    (row as? GemListRow.Text)?.let { ListItemModel(title = it.title.text(context), subtitle = it.value) }
}

internal fun GemRewardsRedemption.uiModel(context: Context): RewardRedemptionUIModel? {
    val asset = option.asset ?: return null
    return RewardRedemptionUIModel(
        redemption = this,
        model = ListItemModel(
            title = context.getString(R.string.rewards_ways_spend_asset_title, value.text()),
            subtitle = points.text(),
            image = ListItemImage.Asset(asset.toPrimitives().id),
        ),
        confirmationMessage = context.getString(R.string.rewards_confirm_redeem, value.text(), points.text()),
    )
}
