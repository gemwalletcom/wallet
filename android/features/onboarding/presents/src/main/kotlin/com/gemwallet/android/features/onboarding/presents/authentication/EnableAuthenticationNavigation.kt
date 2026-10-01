package com.gemwallet.android.features.onboarding.presents.authentication

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import kotlinx.serialization.Serializable

@Serializable
data object EnableAuthenticationRoute : NavKey

fun EntryProviderScope<NavKey>.enableAuthenticationScreen(onEnable: () -> Unit, onSkip: () -> Unit) {
    entry<EnableAuthenticationRoute> {
        EnableAuthenticationScene(onEnable = onEnable, onSkip = onSkip)
    }
}
