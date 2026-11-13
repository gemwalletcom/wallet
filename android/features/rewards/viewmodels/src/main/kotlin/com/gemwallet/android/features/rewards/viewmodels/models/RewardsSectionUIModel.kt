package com.gemwallet.android.features.rewards.viewmodels.models

import android.content.Context
import androidx.compose.runtime.Immutable
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.listItemModel
import com.gemwallet.android.ui.components.list_item.rows
import com.gemwallet.android.ui.localization.string
import uniffi.gemstone.GemRewardsState

@Immutable
data class RewardsSectionUIModel(val title: String?, val rows: List<ListItemModel>)

internal fun GemRewardsState.sectionModels(context: Context): List<RewardsSectionUIModel> = sections.map { section ->
    RewardsSectionUIModel(title = section.title?.string(context), rows = section.rows.mapNotNull { it.listItemModel(context) })
}
