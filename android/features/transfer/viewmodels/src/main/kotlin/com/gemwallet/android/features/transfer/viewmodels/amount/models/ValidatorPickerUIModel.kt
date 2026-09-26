package com.gemwallet.android.features.transfer.viewmodels.amount.models

import uniffi.gemstone.GemStakeValidatorOptions

data class ValidatorPickerUIModel(val selection: GemStakeValidatorOptions, val selectedId: String)
