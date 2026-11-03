package com.gemwallet.android.features.onboarding.presents.authentication

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.ui.rememberAuthorizedAction
import kotlinx.serialization.Serializable

@Serializable
data object EnableAuthenticationRoute : NavKey

fun EntryProviderScope<NavKey>.enableAuthenticationScreen(onEnable: () -> Unit, onSkip: () -> Unit) {
    entry<EnableAuthenticationRoute> {
        EnableAuthenticationScene(onEnable = rememberAuthorizedAction(AuthRequest.Required, onEnable), onSkip = onSkip)
    }
}
