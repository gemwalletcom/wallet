package com.gemwallet.android.features.referral.viewmodels.models

import android.content.Context
import androidx.annotation.StringRes
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.listItemModel
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.localization.titleRes
import uniffi.gemstone.GemRewardsRedemption
import uniffi.gemstone.GemRewardsState

data class RewardRedemptionUIModel(val redemption: GemRewardsRedemption, val model: ListItemModel, val confirmationMessage: String)

data class RewardsSectionUIModel(@StringRes val title: Int?, val rows: List<ListItemModel>)

internal fun GemRewardsState.sectionModels(context: Context): List<RewardsSectionUIModel> = sections.map { section ->
    RewardsSectionUIModel(title = section.title.titleRes(), rows = section.rows.mapNotNull { it.listItemModel(context) })
}

internal fun GemRewardsRedemption.uiModel(context: Context): RewardRedemptionUIModel = RewardRedemptionUIModel(
    redemption = this,
    model = ListItemModel(
        title = title.string(context),
        subtitle = points.text(),
        image = assetId.toAssetId()?.let(ListItemImage::Asset),
    ),
    confirmationMessage = context.getString(R.string.rewards_confirm_redeem, value.text(), points.text()),
)
