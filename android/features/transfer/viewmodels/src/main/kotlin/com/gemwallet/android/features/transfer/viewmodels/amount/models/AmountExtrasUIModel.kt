package com.gemwallet.android.features.transfer.viewmodels.amount.models

import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.wallet.core.primitives.Resource
import uniffi.gemstone.GemPickerOption
import uniffi.gemstone.GemValidatorRow

sealed interface AmountExtrasUIModel {
    data object None : AmountExtrasUIModel

    data class Resources(val options: List<Resource>, val selected: Resource) : AmountExtrasUIModel

    data class Validator(val row: GemValidatorRow, val canSelect: Boolean) : AmountExtrasUIModel

    data class EarnProvider(val row: GemValidatorRow) : AmountExtrasUIModel

    data class Perpetual(val leverage: ListItemModel?, val leverages: List<GemPickerOption>, val selectedLeverage: GemPickerOption?, val autoclose: ListItemModel?) : AmountExtrasUIModel
}
