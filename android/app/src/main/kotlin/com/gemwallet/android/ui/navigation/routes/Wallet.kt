package com.gemwallet.android.ui.navigation.routes

import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.domains.wallet.WalletSecretInput
import com.gemwallet.android.features.create_wallet.views.PhraseAlertDialog
import com.gemwallet.android.features.wallet.presents.WalletImageNavScreen
import com.gemwallet.android.features.wallet.presents.WalletImageSource
import com.gemwallet.android.features.wallet.presents.WalletNavScreen
import com.gemwallet.android.features.wallet.presents.WalletSecretDataNavScreen
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.gemwallet.android.ui.requestAuth
import com.wallet.core.primitives.WalletId
import kotlinx.serialization.Serializable

@Serializable
data class WalletDetailsRoute(val walletId: WalletId) : NavKey

@Serializable
data class WalletImageRoute(val walletId: WalletId, val source: WalletImageSource = WalletImageSource.Wallet) : NavKey

@Serializable
data class WalletSecurityReminderRoute(val input: WalletSecretInput) : NavKey

@Serializable
data class WalletPhraseRoute(val input: WalletSecretInput) : NavKey

fun EntryProviderScope<NavKey>.walletScreen(onBoard: () -> Unit, onCancel: () -> Unit, onSelectImage: (WalletId) -> Unit, onSecurityReminder: (WalletSecretInput) -> Unit, onSecurityReminderAccepted: (WalletSecretInput) -> Unit) {
    entry<WalletDetailsRoute>(
        metadata = { key -> routeArguments(RouteArgument.WalletId to key.walletId.id) },
    ) {
        val context = LocalContext.current

        WalletNavScreen(
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
    ) { key ->
        WalletImageNavScreen(onCancel = onCancel, source = key.source)
    }

    entry<WalletSecurityReminderRoute> { key ->
        PhraseAlertDialog(
            title = stringResource(key.input.kind.stringRes()),
            onAccept = { onSecurityReminderAccepted(key.input) },
            onCancel = onCancel,
        )
    }

    entry<WalletPhraseRoute>(
        metadata = { key ->
            routeArguments(
                RouteArgument.WalletId to key.input.walletId.id,
                RouteArgument.Type to key.input.kind,
            )
        },
    ) {
        WalletSecretDataNavScreen(onCancel = onCancel)
    }
}
