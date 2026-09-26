package com.gemwallet.android.features.wallets.presents

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.wallets.presents.components.WalletAddress
import com.gemwallet.android.features.wallets.presents.dialogs.ConfirmWalletDeleteDialog
import com.gemwallet.android.features.wallets.viewmodels.models.WalletDetailUIModel
import com.gemwallet.android.features.wallets.viewmodels.models.WalletSecretUIModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.GemTextField
import com.gemwallet.android.ui.components.image.WalletAvatar
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.defaultPadding
import com.gemwallet.android.ui.theme.extraLargeIconSize
import com.gemwallet.android.ui.theme.paddingDefault

@Composable
internal fun WalletDetailScene(wallet: WalletDetailUIModel?, secret: WalletSecretUIModel?, snackbar: SnackbarHostState? = null, onAction: (WalletDetailAction) -> Unit) {
    wallet ?: return
    var showDeleteDialog by remember { mutableStateOf(false) }

    var walletName by remember(wallet.name) {
        mutableStateOf(wallet.name)
    }
    Scene(
        title = stringResource(id = R.string.common_wallet),
        snackbar = snackbar,
        actions = {
            TextButton(
                onClick = { onAction(WalletDetailAction.Cancel) },
                colors = ButtonDefaults.textButtonColors()
                    .copy(contentColor = MaterialTheme.colorScheme.onBackground),
            ) {
                Text(stringResource(R.string.common_done).uppercase())
            }
        },
        onClose = { onAction(WalletDetailAction.Cancel) },
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .verticalScroll(rememberScrollState()),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            WalletAvatarHeader(wallet = wallet, onClick = { onAction(WalletDetailAction.SelectImage) })
            GemTextField(
                modifier = Modifier.fillMaxWidth(),
                label = stringResource(id = R.string.wallet_name),
                value = walletName,
                onValueChange = {
                    onAction(WalletDetailAction.SetName(it))
                    walletName = it
                },
                singleLine = true,
            )
            secret?.let {
                ListItem(
                    model = it.model,
                    listPosition = ListPosition.Single,
                    modifier = Modifier.clickable { onAction(WalletDetailAction.ShowPhrase(it.input)) },
                    accessory = { DataBadgeChevron() },
                )
            }
            WalletAddress(wallet.address, wallet.addressExplorer)

            Spacer16()

            Button(
                modifier = Modifier
                    .fillMaxWidth()
                    .defaultPadding(),
                colors = ButtonDefaults.buttonColors()
                    .copy(containerColor = MaterialTheme.colorScheme.error),
                onClick = { showDeleteDialog = true },
            ) {
                Text(text = stringResource(id = R.string.common_delete))
            }
        }
    }

    if (showDeleteDialog) {
        ConfirmWalletDeleteDialog(
            walletName = walletName,
            onConfirm = {
                showDeleteDialog = false
                onAction(WalletDetailAction.Delete)
            },
        ) { showDeleteDialog = false }
    }
}

@Composable
private fun WalletAvatarHeader(wallet: WalletDetailUIModel, onClick: () -> Unit) {
    WalletAvatar(
        imageUrl = wallet.avatar.imageUrl,
        placeholder = wallet.avatar.placeholder,
        size = extraLargeIconSize,
        modifier = Modifier.padding(vertical = paddingDefault),
        supportIcon = R.drawable.ic_edit_badge,
        onClick = onClick,
    )
}
