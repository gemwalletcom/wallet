package com.gemwallet.android.features.stake.viewmodels.models

import android.content.Context
import com.gemwallet.android.ui.components.list_item.DelegationRowUIModel
import com.gemwallet.android.ui.localization.stringRes
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemStakeSection

sealed interface StakeSectionUIModel {
    val title: String

    data class Manage(override val title: String) : StakeSectionUIModel
    data class Resources(override val title: String, val rows: List<GemListRow>) : StakeSectionUIModel
    data class Delegations(override val title: String, val rows: List<DelegationRowUIModel>) : StakeSectionUIModel
}

internal fun GemStakeSection.uiModel(context: Context, rows: List<DelegationRowUIModel>, resourceRows: List<GemListRow>): StakeSectionUIModel {
    val title = context.getString(stringRes())
    return when (this) {
        GemStakeSection.MANAGE -> StakeSectionUIModel.Manage(title)
        GemStakeSection.RESOURCES -> StakeSectionUIModel.Resources(title, resourceRows)
        GemStakeSection.DELEGATIONS -> StakeSectionUIModel.Delegations(title, rows)
    }
}
