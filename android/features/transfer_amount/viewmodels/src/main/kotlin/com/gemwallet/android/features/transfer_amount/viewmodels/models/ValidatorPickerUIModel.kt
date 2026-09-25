package com.gemwallet.android.features.transfer_amount.viewmodels.models

import uniffi.gemstone.GemStakeValidatorSelection

data class ValidatorPickerUIModel(val selection: GemStakeValidatorSelection, val selectedId: String)
