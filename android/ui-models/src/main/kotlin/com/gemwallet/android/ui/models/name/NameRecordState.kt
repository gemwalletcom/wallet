package com.gemwallet.android.ui.models.name

import com.wallet.core.primitives.NameRecord

sealed interface NameRecordState {
    data object None : NameRecordState
    data object Loading : NameRecordState
    data object Error : NameRecordState
    data class Complete(val record: NameRecord) : NameRecordState

    val nameRecord: NameRecord?
        get() = (this as? Complete)?.record
}
