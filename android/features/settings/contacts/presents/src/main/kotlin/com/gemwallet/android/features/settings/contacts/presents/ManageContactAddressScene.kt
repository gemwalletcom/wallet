package com.gemwallet.android.features.settings.contacts.presents

import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ext.asset
import com.gemwallet.android.features.recipient.presents.components.AddressChainField
import com.gemwallet.android.features.recipient.presents.components.MemoTextField
import com.gemwallet.android.features.settings.contacts.viewmodels.models.ContactAddressInput
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.QrCodeScannerModal
import com.gemwallet.android.ui.components.list_item.ChainItem
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition

@Composable
fun ManageContactAddressScene(
    input: ContactAddressInput,
    onAddressChange: (String) -> Unit,
    onMemoChange: (String) -> Unit,
    onScan: (String) -> Unit,
    onPaste: (String) -> Unit,
    onAction: (ManageContactAddressAction) -> Unit,
) {
    var scanning by remember { mutableStateOf(false) }

    Scene(
        title = stringResource(R.string.common_address),
        backHandle = true,
        onClose = { onAction(ManageContactAddressAction.Cancel) },
        actions = {
            IconButton(onClick = { onAction(ManageContactAddressAction.Confirm) }, enabled = input.isConfirmEnabled) {
                Icon(imageVector = AppIcons.Check, contentDescription = "")
            }
        },
    ) {
        SubheaderItem(title = stringResource(R.string.transfer_network))
        ChainItem(
            title = input.chain.asset().name,
            icon = input.chain,
            listPosition = ListPosition.Single,
            trailing = { DataBadgeChevron() },
            onClick = { onAction(ManageContactAddressAction.SelectChain) },
        )

        AddressChainField(
            value = input.address,
            label = stringResource(R.string.common_address),
            state = input.nameResolveState,
            onValueChange = onAddressChange,
            error = if (input.showAddressError) {
                stringResource(R.string.errors_invalid_asset_address, input.chain.asset().name)
            } else {
                ""
            },
            onPaste = onPaste,
            onQrScanner = { scanning = true },
        )

        if (input.showMemo) {
            MemoTextField(
                value = input.memo,
                label = stringResource(R.string.transfer_memo),
                onValueChange = onMemoChange,
            )
        }
    }

    QrCodeScannerModal(
        isVisible = scanning,
        onDismissRequest = { scanning = false },
        onResult = {
            onScan(it)
            scanning = false
        },
    )
}
