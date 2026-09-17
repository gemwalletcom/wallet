package com.gemwallet.android.features.stake.viewmodels.models

import android.content.Context
import com.gemwallet.android.features.stake.viewmodels.localization.stringRes
import com.gemwallet.android.ui.components.list_item.ValidatorRowUIModel
import com.gemwallet.android.ui.components.list_item.uiModel
import com.wallet.core.primitives.Delegation
import uniffi.gemstone.GemStakeSection
import uniffi.gemstone.GemValidatorRow

sealed interface StakeSectionUIModel {
    val title: String

    data class Manage(override val title: String) : StakeSectionUIModel
    data class Resources(override val title: String) : StakeSectionUIModel
    data class Delegations(override val title: String, val rows: List<StakeDelegationRowUIModel>) : StakeSectionUIModel
}

data class StakeDelegationRowUIModel(
    val delegation: Delegation,
    val validator: ValidatorRowUIModel,
)

internal fun GemStakeSection.uiModel(
    context: Context,
    delegations: List<Delegation>,
    validatorRows: Map<String, GemValidatorRow>,
): StakeSectionUIModel {
    val title = context.getString(stringRes())
    return when (this) {
        GemStakeSection.MANAGE -> StakeSectionUIModel.Manage(title)
        GemStakeSection.RESOURCES -> StakeSectionUIModel.Resources(title)
        GemStakeSection.DELEGATIONS -> StakeSectionUIModel.Delegations(
            title = title,
            rows = delegations.mapNotNull { delegation ->
                validatorRows[delegation.validator.id]?.let { StakeDelegationRowUIModel(delegation, it.uiModel()) }
            },
        )
    }
}
