package com.gemwallet.android.features.onboarding.presents.import_wallet

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.model.ImportType
import com.wallet.core.primitives.Chain
import kotlinx.serialization.Serializable
import uniffi.gemstone.GemWalletImportKind

@Serializable
data object ImportWalletTypeRoute : NavKey

@Serializable
sealed interface ImportWalletRoute : NavKey {
    @Serializable
    data object MulticoinWallet : ImportWalletRoute

    @Serializable
    data class ChainWallet(val kind: GemWalletImportKind, val chain: Chain) : ImportWalletRoute
}

fun EntryProviderScope<NavKey>.importWalletScreen(onCancel: () -> Unit, onImported: () -> Unit, onSelectType: (ImportType) -> Unit) {
    entry<ImportWalletTypeRoute> {
        ImportWalletTypeScreen(onClose = onCancel, onSelect = onSelectType)
    }
    entry<ImportWalletRoute.MulticoinWallet> {
        ImportWalletScreen(
            importType = ImportType(GemWalletImportKind.PHRASE),
            onCancel = onCancel,
            onImported = onImported,
        )
    }

    entry<ImportWalletRoute.ChainWallet> { key ->
        ImportWalletScreen(
            importType = ImportType(key.kind, key.chain),
            onCancel = onCancel,
            onImported = onImported,
        )
    }
}
