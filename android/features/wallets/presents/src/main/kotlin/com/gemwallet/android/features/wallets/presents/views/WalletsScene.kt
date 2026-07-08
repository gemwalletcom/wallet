package com.gemwallet.android.features.wallets.presents.views

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.domains.wallet.aggregates.WalletDataAggregate
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.features.wallets.presents.views.components.WalletsActions
import com.gemwallet.android.features.wallets.presents.views.components.wallets

@Composable
internal fun WalletsScene(
    pinnedWallets: List<WalletDataAggregate>,
    unpinnedWallets: List<WalletDataAggregate>,
    onAction: (WalletsAction) -> Unit,
) {
    val longPressedWallet = remember {
        mutableStateOf("")
    }

    Scene(
        title = stringResource(id = R.string.wallets_title),
        onClose = { onAction(WalletsAction.Cancel) },
    ) {
        LazyColumn(modifier = Modifier.fillMaxSize()) {
            item {
                WalletsActions(
                    onCreate = { onAction(WalletsAction.Create) },
                    onImport = { onAction(WalletsAction.Import) },
                )
            }
            wallets(
                wallets = pinnedWallets,
                longPressedWallet = longPressedWallet,
                onEdit = { onAction(WalletsAction.Edit(it)) },
                onSelectWallet = { onAction(WalletsAction.Select(it)) },
                onDeleteWallet = { onAction(WalletsAction.Delete(it)) },
                onTogglePin = { onAction(WalletsAction.TogglePin(it)) },
                isPinned = true,
            )
            wallets(
                wallets = unpinnedWallets,
                longPressedWallet = longPressedWallet,
                onEdit = { onAction(WalletsAction.Edit(it)) },
                onSelectWallet = { onAction(WalletsAction.Select(it)) },
                onDeleteWallet = { onAction(WalletsAction.Delete(it)) },
                onTogglePin = { onAction(WalletsAction.TogglePin(it)) },
            )
        }
    }
}
