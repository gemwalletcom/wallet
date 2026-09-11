package com.gemwallet.android.ui.navigation.routes

import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.create_wallet.views.KeyAlertDialog
import com.gemwallet.android.features.create_wallet.views.PhraseAlertDialog
import com.gemwallet.android.features.wallet.presents.WalletImageNavScreen
import com.gemwallet.android.features.wallet.presents.WalletImageSource
import com.gemwallet.android.features.wallet.presents.WalletNavScreen
import com.gemwallet.android.features.wallet.presents.WalletPrivateKeyChainNavScreen
import com.gemwallet.android.features.wallet.presents.WalletSecretDataNavScreen
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.gemwallet.android.ui.requestAuth
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletId
import com.gemwallet.android.ui.components.list_item.titleRes
import uniffi.gemstone.GemWalletSecretKind
import kotlinx.serialization.Serializable


@Serializable
data class WalletDetailsRoute(val walletId: WalletId) : NavKey

@Serializable
data class WalletImageRoute(
    val walletId: WalletId,
    val source: WalletImageSource = WalletImageSource.Wallet,
) : NavKey

@Serializable
data class WalletPrivateKeyChainRoute(val walletId: WalletId, val chains: List<Chain>) : NavKey

@Serializable
data class WalletSecurityReminderRoute(val walletId: WalletId, val secretKind: GemWalletSecretKind, val chain: Chain? = null) : NavKey

@Serializable
data class WalletPhraseRoute(val walletId: WalletId, val secretKind: GemWalletSecretKind, val chain: Chain? = null) : NavKey

fun EntryProviderScope<NavKey>.walletScreen(
    onBoard: () -> Unit,
    onCancel: () -> Unit,
    onSelectImage: (WalletId) -> Unit,
    onPrivateKeyChains: (WalletId, List<Chain>) -> Unit,
    onSecurityReminder: (WalletId, GemWalletSecretKind, Chain?) -> Unit,
    onSecurityReminderAccepted: (WalletId, GemWalletSecretKind, Chain?) -> Unit,
) {
    entry<WalletDetailsRoute>(
        metadata = { key -> routeArguments(RouteArgument.WalletId to key.walletId.id) },
    ) {
        val context = LocalContext.current

        WalletNavScreen(
            onPhraseShow = { walletId, secretKind ->
                context.requestAuth(AuthRequest.Default) { onSecurityReminder(walletId, secretKind, null) }
            },
            onPrivateKeyShow = { walletId, chains ->
                context.requestAuth(AuthRequest.Default) {
                    when (chains.size) {
                        1 -> onSecurityReminder(walletId, GemWalletSecretKind.PRIVATE_KEY, chains.first())
                        else -> onPrivateKeyChains(walletId, chains)
                    }
                }
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

    entry<WalletPrivateKeyChainRoute> { key ->
        WalletPrivateKeyChainNavScreen(
            chains = key.chains,
            onSelect = { chain -> onSecurityReminder(key.walletId, GemWalletSecretKind.PRIVATE_KEY, chain) },
            onCancel = onCancel,
        )
    }

    entry<WalletSecurityReminderRoute> { key ->
        val onAccept = { onSecurityReminderAccepted(key.walletId, key.secretKind, key.chain) }
        when (val chain = key.chain) {
            null -> PhraseAlertDialog(
                title = stringResource(key.secretKind.titleRes),
                onAccept = onAccept,
                onCancel = onCancel,
            )
            else -> KeyAlertDialog(
                chain = chain,
                onAccept = onAccept,
                onCancel = onCancel,
            )
        }
    }

    entry<WalletPhraseRoute>(
        metadata = { key ->
            routeArguments(
                RouteArgument.WalletId to key.walletId.id,
                RouteArgument.Type to key.secretKind,
                RouteArgument.Chain to key.chain?.string,
            )
        },
    ) {
        WalletSecretDataNavScreen(onCancel = onCancel)
    }
}
