package com.gemwallet.android.features.onboarding

import androidx.annotation.Keep
import androidx.navigation.NavController
import androidx.navigation.NavGraphBuilder
import androidx.navigation.compose.composable
import androidx.navigation.navOptions
import androidx.navigation.toRoute
import kotlinx.serialization.Serializable

@Keep
@Serializable
enum class AcceptTermsDestination {
    Create,
    Import,
}

@Serializable
data class AcceptTermsRoute(val destination: AcceptTermsDestination)

fun NavController.navigateToAcceptTerms(destination: AcceptTermsDestination) {
    navigate(AcceptTermsRoute(destination), navOptions { launchSingleTop = true })
}

fun NavGraphBuilder.acceptTermsScreen(
    onCancel: () -> Unit,
    onAccept: (AcceptTermsDestination) -> Unit,
) {
    composable<AcceptTermsRoute> { entry ->
        val route = entry.toRoute<AcceptTermsRoute>()
        AcceptTermsScreen(
            onCancel = onCancel,
            onAccept = { onAccept(route.destination) },
        )
    }
}
