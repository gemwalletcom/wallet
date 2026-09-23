package com.gemwallet.android.features.settings.contacts.presents

import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ext.networkName
import com.gemwallet.android.features.settings.contacts.viewmodels.models.ContactAddressInput
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.QrCodeScannerModal
import com.gemwallet.android.ui.components.fields.AddressChainField
import com.gemwallet.android.ui.components.fields.MemoTextField
import com.gemwallet.android.ui.components.list_item.ChainItem
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.wallet.core.primitives.QRScanType

@Composable
fun ContactAddressEditorScene(input: ContactAddressInput, onAddressChange: (String) -> Unit, onMemoChange: (String) -> Unit, onScan: (String) -> Unit, onPaste: (String) -> Unit, onAction: (ContactAddressEditorAction) -> Unit) {
    var scanning by remember { mutableStateOf(false) }

    Scene(
        title = stringResource(R.string.common_address),
        onClose = { onAction(ContactAddressEditorAction.Cancel) },
        actions = {
            IconButton(onClick = { onAction(ContactAddressEditorAction.Confirm) }, enabled = input.isConfirmEnabled) {
                Icon(imageVector = AppIcons.Check, contentDescription = "")
            }
        },
    ) {
        SubheaderItem(title = stringResource(R.string.transfer_network))
        ChainItem(
            title = input.chain.networkName(),
            icon = input.chain,
            listPosition = ListPosition.Single,
            trailing = { DataBadgeChevron() },
            onClick = { onAction(ContactAddressEditorAction.SelectChain) },
        )
        AddressChainField(
            value = input.address,
            label = stringResource(R.string.common_address),
            indicator = input.nameResolveIndicator,
            onValueChange = onAddressChange,
            error = input.addressError,
            onPaste = onPaste,
            onQrScanner = { scanning = true },
        )
        if (input.showsMemo) {
            MemoTextField(
                value = input.memo,
                label = stringResource(R.string.transfer_memo),
                onValueChange = onMemoChange,
            )
        }
    }

    QrCodeScannerModal(
        isVisible = scanning,
        scanType = QRScanType.Address,
        onDismissRequest = { scanning = false },
        onResult = {
            onScan(it)
            scanning = false
        },
    )
}
