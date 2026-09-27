package com.gemwallet.android.features.wallets.presents

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.wallets.presents.components.WalletsActions
import com.gemwallet.android.features.wallets.presents.components.wallets
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.Scene
import uniffi.gemstone.GemWalletSection

@Composable
internal fun WalletsScene(sections: List<GemWalletSection>, snackbar: SnackbarHostState? = null, onAction: (WalletsAction) -> Unit) {
    val longPressedWallet = remember {
        mutableStateOf("")
    }

    Scene(
        title = stringResource(id = R.string.wallets_title),
        snackbar = snackbar,
        onClose = { onAction(WalletsAction.Cancel) },
    ) {
        LazyColumn(modifier = Modifier.fillMaxSize()) {
            item {
                WalletsActions(
                    onCreate = { onAction(WalletsAction.Create) },
                    onImport = { onAction(WalletsAction.Import) },
                )
            }
            sections.forEach { section ->
                wallets(
                    section = section,
                    longPressedWallet = longPressedWallet,
                    onEdit = { onAction(WalletsAction.Edit(it)) },
                    onSelectWallet = { onAction(WalletsAction.Select(it)) },
                    onDeleteWallet = { onAction(WalletsAction.Delete(it)) },
                    onTogglePin = { onAction(WalletsAction.TogglePin(it)) },
                )
            }
        }
    }
}
