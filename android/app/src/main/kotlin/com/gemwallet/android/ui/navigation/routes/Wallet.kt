package com.gemwallet.android.ui.navigation.routes

import androidx.compose.ui.platform.LocalContext
import androidx.navigation.NavController
import androidx.navigation.NavGraphBuilder
import androidx.navigation.NavOptions
import androidx.navigation.compose.composable
import androidx.navigation.navOptions
import androidx.navigation.toRoute
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.create_wallet.views.PhraseAlertDialog
import com.gemwallet.android.features.wallet.presents.WalletNavScreen
import com.gemwallet.android.features.wallet.presents.WalletSecretDataNavScreen
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.requestAuth
import com.wallet.core.primitives.WalletType
import kotlinx.serialization.Serializable


@Serializable
data class WalletDetailsRoute(val walletId: String)

@Serializable
data class WalletSecurityReminderRoute(val walletId: String, val type: WalletType)

@Serializable
data class WalletPhraseRoute(val walletId: String, val type: WalletType)

fun NavController.navigateToWalletScreen(walletId: String, navOptions: NavOptions? = null) {
    navigate(WalletDetailsRoute(walletId), navOptions ?: navOptions {launchSingleTop = true})
}

fun NavController.navigateToSecurityReminderScreen(walletId: String, type: WalletType, navOptions: NavOptions? = null) {
    navigate(WalletSecurityReminderRoute(walletId, type), navOptions ?: navOptions {launchSingleTop = true})
}

fun NavController.navigateToPhraseScreen(walletId: String, type: WalletType, navOptions: NavOptions? = null) {
    navigate(WalletPhraseRoute(walletId, type), navOptions ?: navOptions {launchSingleTop = true})
}

fun NavGraphBuilder.walletScreen(
    onBoard: () -> Unit,
    onCancel: () -> Unit,
    onSecurityReminder: (String, WalletType) -> Unit,
    onPhraseShow: (String, WalletType) -> Unit,
) {
    composable<WalletDetailsRoute> {
        val context = LocalContext.current

        WalletNavScreen(
            onPhraseShow = { walletId, type ->
                context.requestAuth(AuthRequest.Phrase) { onSecurityReminder(walletId, type) }
            },
            onBoard = onBoard,
            onCancel = onCancel,
        )
    }

    composable<WalletSecurityReminderRoute> { backStackEntry ->
        val route = backStackEntry.toRoute<WalletSecurityReminderRoute>()
        val title = when (route.type) {
            WalletType.PrivateKey -> stringResource(R.string.common_private_key)
            else -> stringResource(R.string.common_secret_phrase)
        }
        PhraseAlertDialog(
            title = title,
            onAccept = { onPhraseShow(route.walletId, route.type) },
            onCancel = onCancel,
        )
    }

    composable<WalletPhraseRoute> {
        WalletSecretDataNavScreen(onCancel = onCancel)
    }
}
