package com.gemwallet.android.features.setup_wallet.navigation

import androidx.navigation.NavController
import androidx.navigation.NavGraphBuilder
import androidx.navigation.NavOptions
import androidx.navigation.compose.composable
import androidx.navigation.navOptions
import com.gemwallet.android.features.setup_wallet.views.SetupWalletScreen
import kotlinx.serialization.Serializable

@Serializable
data class SetupWalletRoute(val walletId: String)

fun NavController.navigateToSetupWalletScreen(walletId: String, navOptions: NavOptions? = null) {
    navigate(SetupWalletRoute(walletId), navOptions ?: navOptions { launchSingleTop = true })
}

fun NavGraphBuilder.setupWalletScreen(
    onComplete: () -> Unit,
) {
    composable<SetupWalletRoute> {
        SetupWalletScreen(onComplete = onComplete)
    }
}
