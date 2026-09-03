package com.gemwallet.android.domains.confirm

sealed interface ConfirmState {
    data object Prepare : ConfirmState
    data object Ready : ConfirmState
    data object Sending : ConfirmState
    class Result(val transactionHash: String) : ConfirmState
    class Error(val error: Throwable) : ConfirmState
    class BroadcastError(val error: Throwable) : ConfirmState
    class FatalError(val messageRes: Int) : ConfirmState
}