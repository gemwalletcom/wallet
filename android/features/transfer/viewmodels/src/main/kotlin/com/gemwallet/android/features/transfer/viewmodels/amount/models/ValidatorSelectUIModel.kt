package com.gemwallet.android.features.transfer.viewmodels.amount.models

import uniffi.gemstone.GemStakeValidatorOptions

data class ValidatorSelectUIModel(val selection: GemStakeValidatorOptions, val selectedId: String)
