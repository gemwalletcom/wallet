package com.gemwallet.android.features.import_wallet.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.cases.wallet.WalletImportResult
import com.gemwallet.android.features.import_wallet.views.ImportScreen
import com.gemwallet.android.features.import_wallet.views.SelectImportTypeScreen
import com.gemwallet.android.model.ImportType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletType
import kotlinx.serialization.Serializable

@Serializable
data object ImportSelectTypeRoute : NavKey

@Serializable
data object ImportMulticoinWalletRoute : NavKey

@Serializable
data class ImportChainWalletRoute(val walletType: WalletType, val chain: Chain) : NavKey

fun EntryProviderScope<NavKey>.importWalletScreen(
    onCancel: () -> Unit,
    onImported: (WalletImportResult) -> Unit,
    onSelectType: (ImportType) -> Unit,
) {
    entry<ImportSelectTypeRoute> {
        SelectImportTypeScreen(onClose = onCancel, onSelect = onSelectType)
    }
    entry<ImportMulticoinWalletRoute> {
        ImportScreen(
            importType = ImportType(WalletType.Multicoin),
            onCancel = onCancel,
            onImported = onImported,
        )
    }

    entry<ImportChainWalletRoute> { key ->
        ImportScreen(
            importType = ImportType(key.walletType, key.chain),
            onCancel = onCancel,
            onImported = onImported,
        )
    }
}
