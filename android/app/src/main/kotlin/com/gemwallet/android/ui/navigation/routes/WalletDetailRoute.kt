package com.gemwallet.android.ui.navigation.routes

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.res.stringResource
import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.domains.wallet.PrivateKeyExportInput
import com.gemwallet.android.domains.wallet.WalletSecretInput
import com.gemwallet.android.features.onboarding.presents.create_wallet.SecurityReminderScene
import com.gemwallet.android.features.wallet.presents.PrivateKeyAccountsScreen
import com.gemwallet.android.features.wallet.presents.SecretDataScreen
import com.gemwallet.android.features.wallet.presents.WalletDetailScreen
import com.gemwallet.android.features.wallet.presents.WalletImageScreen
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.gemwallet.android.ui.rememberAuthorizedAction
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletId
import kotlinx.serialization.Serializable
import uniffi.gemstone.GemSecurityReminderKind
import uniffi.gemstone.GemWalletSecretKind

@Serializable
data class WalletDetailRoute(val walletId: WalletId) : NavKey

@Serializable
data class WalletImageRoute(val walletId: WalletId) : NavKey

@Serializable
data class SecurityReminderRoute(val input: WalletSecretInput) : NavKey

@Serializable
data class ExportWalletRoute(val input: WalletSecretInput) : NavKey

@Serializable
data class PrivateKeyWarningRoute(val input: PrivateKeyExportInput) : NavKey

@Serializable
data class PrivateKeyAccountsRoute(val walletId: WalletId) : NavKey

@Serializable
data class ExportPrivateKeyRoute(val walletId: WalletId, val chain: Chain) : NavKey

fun EntryProviderScope<NavKey>.walletDetailScreen(
    onBoard: () -> Unit,
    onCancel: () -> Unit,
    onSelectImage: (WalletId) -> Unit,
    onSecurityReminder: (WalletSecretInput) -> Unit,
    onSecurityReminderAccepted: (WalletSecretInput) -> Unit,
    onPrivateKeyWarning: (PrivateKeyExportInput) -> Unit,
    onPrivateKeyWarningAccepted: (PrivateKeyExportInput) -> Unit,
    onPrivateKeyAccount: (WalletId, Chain) -> Unit,
) {
    entry<WalletDetailRoute>(
        metadata = { key -> routeArguments(RouteArgument.WalletId to key.walletId.id) },
    ) {
        WalletDetailScreen(
            onPhraseShow = onSecurityReminder,
            onPrivateKeyShow = onPrivateKeyWarning,
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
            kind = key.input.kind.securityReminderKind(),
            onAccept = rememberAuthorizedAction(AuthRequest.Default) { onSecurityReminderAccepted(key.input) },
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

    entry<PrivateKeyWarningRoute> { key ->
        val onAuthorized = rememberAuthorizedAction(AuthRequest.Default) { onPrivateKeyWarningAccepted(key.input) }

        SecurityReminderScene(
            title = stringResource(R.string.common_private_key),
            kind = GemSecurityReminderKind.ACCOUNT_KEY,
            onAccept = {
                when (key.input.chain) {
                    null -> onPrivateKeyWarningAccepted(key.input)
                    else -> onAuthorized()
                }
            },
            onCancel = onCancel,
        )
    }

    entry<PrivateKeyAccountsRoute>(
        metadata = { key -> routeArguments(RouteArgument.WalletId to key.walletId.id) },
    ) { key ->
        var selectedChain by rememberSaveable { mutableStateOf<Chain?>(null) }
        val onAuthorized = rememberAuthorizedAction(AuthRequest.Default) {
            selectedChain?.let { onPrivateKeyAccount(key.walletId, it) }
        }

        PrivateKeyAccountsScreen(
            onSelect = { chain ->
                selectedChain = chain
                onAuthorized()
            },
            onCancel = onCancel,
        )
    }

    entry<ExportPrivateKeyRoute>(
        metadata = { key ->
            routeArguments(
                RouteArgument.WalletId to key.walletId.id,
                RouteArgument.Type to GemWalletSecretKind.PRIVATE_KEY,
                RouteArgument.Chain to key.chain.string,
            )
        },
    ) {
        SecretDataScreen(onCancel = onCancel)
    }
}

private fun GemWalletSecretKind.securityReminderKind(): GemSecurityReminderKind = when (this) {
    GemWalletSecretKind.PHRASE -> GemSecurityReminderKind.PHRASE
    GemWalletSecretKind.PRIVATE_KEY -> GemSecurityReminderKind.PRIVATE_KEY
}
