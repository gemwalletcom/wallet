package com.gemwallet.android.features.transfer_amount.models

import com.wallet.core.primitives.DelegationValidator

sealed interface ValidatorsUIState {
    data object Loading : ValidatorsUIState

    data object Empty : ValidatorsUIState

    class Loaded(
        val recommended: List<DelegationValidator>,
        val validators: List<DelegationValidator>,
        val loading: Boolean = false,
    ) : ValidatorsUIState
}