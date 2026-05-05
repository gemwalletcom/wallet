package com.gemwallet.android.features.onboarding

import androidx.annotation.Keep
import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import kotlinx.serialization.Serializable

@Keep
@Serializable
enum class AcceptTermsDestination {
    Create,
    Import,
}

@Serializable
data class AcceptTermsRoute(val destination: AcceptTermsDestination) : NavKey

fun EntryProviderScope<NavKey>.acceptTermsScreen(
    onCancel: () -> Unit,
    onAccept: (AcceptTermsDestination) -> Unit,
) {
    entry<AcceptTermsRoute> { route ->
        AcceptTermsScreen(
            onCancel = onCancel,
            onAccept = { onAccept(route.destination) },
        )
    }
}
