package com.gemwallet.android.features.transfer_amount.viewmodels.models

import com.gemwallet.android.ui.components.list_item.ValidatorRowUIModel
import com.gemwallet.android.ui.components.list_item.uiModel
import uniffi.gemstone.GemStakeValidatorSelection

data class ValidatorsUIModel(val recommended: List<ValidatorRowUIModel>, val options: List<ValidatorRowUIModel>)

internal fun GemStakeValidatorSelection.uiModel(): ValidatorsUIModel = ValidatorsUIModel(
    recommended = recommended.map { it.uiModel() },
    options = options.map { it.uiModel() },
)
