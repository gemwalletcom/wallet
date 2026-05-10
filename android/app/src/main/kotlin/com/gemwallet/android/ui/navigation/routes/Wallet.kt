package com.gemwallet.android.ui.navigation.routes

import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.create_wallet.views.PhraseAlertDialog
import com.gemwallet.android.features.wallet.presents.WalletNavScreen
import com.gemwallet.android.features.wallet.presents.WalletSecretDataNavScreen
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.gemwallet.android.ui.requestAuth
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletType
import kotlinx.serialization.Serializable


@Serializable
data class WalletDetailsRoute(val walletId: WalletId) : NavKey

@Serializable
data class WalletSecurityReminderRoute(val walletId: WalletId, val type: WalletType) : NavKey

@Serializable
data class WalletPhraseRoute(val walletId: WalletId, val type: WalletType) : NavKey

fun EntryProviderScope<NavKey>.walletScreen(
    onBoard: () -> Unit,
    onCancel: () -> Unit,
    onSecurityReminder: (WalletId, WalletType) -> Unit,
    onSecurityReminderAccepted: (WalletId, WalletType) -> Unit,
) {
    entry<WalletDetailsRoute>(
        metadata = { key -> routeArguments(RouteArgument.WalletId to key.walletId.id) },
    ) {
        val context = LocalContext.current

        WalletNavScreen(
            onPhraseShow = { walletId, type ->
                context.requestAuth(AuthRequest.Default) { onSecurityReminder(walletId, type) }
            },
            onBoard = onBoard,
            onCancel = onCancel,
        )
    }

    entry<WalletSecurityReminderRoute> { key ->
        val title = when (key.type) {
            WalletType.PrivateKey -> stringResource(R.string.common_private_key)
            else -> stringResource(R.string.common_secret_phrase)
        }
        PhraseAlertDialog(
            title = title,
            onAccept = { onSecurityReminderAccepted(key.walletId, key.type) },
            onCancel = onCancel,
        )
    }

    entry<WalletPhraseRoute>(
        metadata = { key ->
            routeArguments(
                RouteArgument.WalletId to key.walletId.id,
                RouteArgument.Type to key.type,
            )
        },
    ) {
        WalletSecretDataNavScreen(onCancel = onCancel)
    }
}
