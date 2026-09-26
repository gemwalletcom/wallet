package com.gemwallet.android.ui.navigation.routes

import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.domains.wallet.WalletSecretInput
import com.gemwallet.android.features.onboarding.presents.create_wallet.SecurityReminderScene
import com.gemwallet.android.features.wallets.presents.SecretDataScreen
import com.gemwallet.android.features.wallets.presents.WalletDetailScreen
import com.gemwallet.android.features.wallets.presents.WalletImageScreen
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.gemwallet.android.ui.requestAuth
import com.wallet.core.primitives.WalletId
import kotlinx.serialization.Serializable

@Serializable
data class WalletDetailRoute(val walletId: WalletId) : NavKey

@Serializable
data class WalletImageRoute(val walletId: WalletId) : NavKey

@Serializable
data class SecurityReminderRoute(val input: WalletSecretInput) : NavKey

@Serializable
data class ExportWalletRoute(val input: WalletSecretInput) : NavKey

fun EntryProviderScope<NavKey>.walletDetailScreen(onBoard: () -> Unit, onCancel: () -> Unit, onSelectImage: (WalletId) -> Unit, onSecurityReminder: (WalletSecretInput) -> Unit, onSecurityReminderAccepted: (WalletSecretInput) -> Unit) {
    entry<WalletDetailRoute>(
        metadata = { key -> routeArguments(RouteArgument.WalletId to key.walletId.id) },
    ) {
        val context = LocalContext.current

        WalletDetailScreen(
            onPhraseShow = { input ->
                context.requestAuth(AuthRequest.Default) { onSecurityReminder(input) }
            },
            onSelectImage = onSelectImage,
            onBoard = onBoard,
            onCancel = onCancel,
        )
    }

    entry<WalletImageRoute>(
        metadata = { key -> routeArguments(RouteArgument.WalletId to key.walletId.id) },
    ) {
        WalletImageScreen(onCancel = onCancel)
    }

    entry<SecurityReminderRoute> { key ->
        SecurityReminderScene(
            title = stringResource(key.input.kind.stringRes()),
            onAccept = { onSecurityReminderAccepted(key.input) },
            onCancel = onCancel,
        )
    }

    entry<ExportWalletRoute>(
        metadata = { key ->
            routeArguments(
                RouteArgument.WalletId to key.input.walletId.id,
                RouteArgument.Type to key.input.kind,
            )
        },
    ) {
        SecretDataScreen(onCancel = onCancel)
    }
}
