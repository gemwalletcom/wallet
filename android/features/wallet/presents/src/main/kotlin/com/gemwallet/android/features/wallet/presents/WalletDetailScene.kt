package com.gemwallet.android.features.wallet.presents

import androidx.annotation.StringRes
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
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.wallet.presents.components.WalletAddress
import com.gemwallet.android.features.wallet.presents.dialogs.ConfirmWalletDeleteDialog
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.GemTextField
import com.gemwallet.android.ui.components.image.WalletAvatar
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemImageStyle
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.style.listItemImage
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.defaultPadding
import com.gemwallet.android.ui.theme.extraLargeIconSize
import com.gemwallet.android.ui.theme.paddingDefault
import uniffi.gemstone.GemSecretRow
import uniffi.gemstone.GemWalletDetails

@Composable
internal fun WalletDetailScene(wallet: GemWalletDetails?, snackbar: SnackbarHostState? = null, onAction: (WalletDetailAction) -> Unit) {
    wallet ?: return
    var showDeleteDialog by remember { mutableStateOf(false) }

    var walletName by rememberSaveable { mutableStateOf(wallet.row.name) }
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
                modifier = Modifier.fillMaxWidth().testTag("nameInputField"),
                label = stringResource(id = R.string.wallet_name),
                value = walletName,
                onValueChange = {
                    onAction(WalletDetailAction.SetName(it))
                    walletName = it
                },
                singleLine = true,
            )
            SecretSection(title = R.string.wallet_backup, row = wallet.backup, onAction = onAction)
            SecretSection(title = R.string.wallet_advanced, row = wallet.advanced, onAction = onAction)
            WalletAddress(wallet.address)

            Spacer16()

            Button(
                modifier = Modifier
                    .fillMaxWidth()
                    .defaultPadding()
                    .testTag("delete"),
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
            prompt = wallet.row.deletePrompt.string(LocalContext.current),
            onConfirm = {
                showDeleteDialog = false
                onAction(WalletDetailAction.Delete)
            },
        ) { showDeleteDialog = false }
    }
}

@Composable
private fun SecretSection(@StringRes title: Int, row: GemSecretRow?, onAction: (WalletDetailAction) -> Unit) {
    row ?: return
    SubheaderItem(title)
    ListItem(
        model = ListItemModel(title = row.title.string(LocalContext.current)),
        listPosition = ListPosition.Single,
        modifier = Modifier.clickable { onAction(WalletDetailAction.ShowSecret(row.export)) },
        accessory = { DataBadgeChevron() },
    )
}

@Composable
private fun WalletAvatarHeader(wallet: GemWalletDetails, onClick: () -> Unit) {
    WalletAvatar(
        image = wallet.row.listItemImage(),
        size = extraLargeIconSize,
        modifier = Modifier.padding(vertical = paddingDefault),
        badgeImage = ListItemImage.Drawable(R.drawable.ic_edit_badge, style = ListItemImageStyle.Avatar),
        onClick = onClick,
    )
}
