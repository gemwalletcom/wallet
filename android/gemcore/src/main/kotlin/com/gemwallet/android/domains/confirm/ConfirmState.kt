package com.gemwallet.android.domains.confirm

sealed interface ConfirmState {
    data object Prepare : ConfirmState
    data object Ready : ConfirmState
    data object Sending : ConfirmState
    class Result(val transactionHash: String, val error: ConfirmError? = null) : ConfirmState
    class Error(val message: ConfirmError) : ConfirmState
    class BroadcastError(val message: ConfirmError) : ConfirmState
    class FatalError(val message: String) : ConfirmState
}