package com.gemwallet.android.features.rewards.viewmodels.models

import android.content.Context
import androidx.annotation.StringRes
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.listItemModel
import com.gemwallet.android.ui.localization.titleRes
import uniffi.gemstone.GemRewardsState

data class RewardsSectionUIModel(@StringRes val title: Int?, val rows: List<ListItemModel>)

internal fun GemRewardsState.sectionModels(context: Context): List<RewardsSectionUIModel> = sections.map { section ->
    RewardsSectionUIModel(title = section.title.titleRes(), rows = section.rows.mapNotNull { it.listItemModel(context) })
}
