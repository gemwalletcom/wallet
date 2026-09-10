package com.gemwallet.android.features.earn.delegation.models

import com.wallet.core.primitives.DelegationValidator
import uniffi.gemstone.GemDelegationCompletion
import uniffi.gemstone.GemDelegationStatus

sealed interface DelegationProperty {
    class Name(val data: String, val url: String?) : DelegationProperty

    class Apr(val data: DelegationValidator) : DelegationProperty

    class State(val completion: GemDelegationCompletion, val availableIn: String) : DelegationProperty

    class TransactionStatus(val status: GemDelegationStatus) : DelegationProperty
}