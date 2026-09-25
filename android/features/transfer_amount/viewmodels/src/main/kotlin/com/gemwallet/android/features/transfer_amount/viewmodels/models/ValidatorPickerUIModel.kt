package com.gemwallet.android.features.transfer_amount.viewmodels.models

import uniffi.gemstone.GemStakeValidatorOptions

data class ValidatorPickerUIModel(val selection: GemStakeValidatorOptions, val selectedId: String)
