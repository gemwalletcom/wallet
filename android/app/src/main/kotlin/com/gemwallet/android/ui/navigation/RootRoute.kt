package com.gemwallet.android.ui.navigation

import androidx.navigation.NavBackStackEntry
import androidx.navigation.NavController

const val walletRootRoute = "/"
const val walletResetArg = "reset"

object NavigationResult {
    const val TOAST_MESSAGE = "toast_message"
}

fun NavController.popBackWithResult(key: String, value: String) {
    previousBackStackEntry?.savedStateHandle?.set(key, value)
    popBackStack()
}

fun NavBackStackEntry.getToastMessage(): String? =
    savedStateHandle.get<String>(NavigationResult.TOAST_MESSAGE)

fun NavBackStackEntry.clearToastMessage() {
    savedStateHandle.remove<String>(NavigationResult.TOAST_MESSAGE)
}
