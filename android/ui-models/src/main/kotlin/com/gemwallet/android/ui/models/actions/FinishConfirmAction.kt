package com.gemwallet.android.ui.models.actions

fun interface FinishConfirmAction {
    operator fun invoke(hash: String)
}
