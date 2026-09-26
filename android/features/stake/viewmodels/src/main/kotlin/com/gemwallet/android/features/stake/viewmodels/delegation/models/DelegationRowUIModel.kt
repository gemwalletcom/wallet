package com.gemwallet.android.features.stake.viewmodels.delegation.models

import uniffi.gemstone.GemListRow

sealed interface DelegationRowUIModel {
    data class Row(val row: GemListRow) : DelegationRowUIModel
    data object Rewards : DelegationRowUIModel
}
