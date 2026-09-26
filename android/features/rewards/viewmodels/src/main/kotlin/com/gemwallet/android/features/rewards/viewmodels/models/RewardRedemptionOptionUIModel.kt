package com.gemwallet.android.features.rewards.viewmodels.models

import android.content.Context
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.localization.string
import uniffi.gemstone.GemRewardsRedemption

data class RewardRedemptionOptionUIModel(val redemption: GemRewardsRedemption, val model: ListItemModel, val confirmationMessage: String)

internal fun GemRewardsRedemption.uiModel(context: Context): RewardRedemptionOptionUIModel = RewardRedemptionOptionUIModel(
    redemption = this,
    model = ListItemModel(
        title = title.string(context),
        subtitle = points.text(),
        image = ListItemImage.Asset(icon),
    ),
    confirmationMessage = context.getString(R.string.rewards_confirm_redeem, value.text(), points.text()),
)
