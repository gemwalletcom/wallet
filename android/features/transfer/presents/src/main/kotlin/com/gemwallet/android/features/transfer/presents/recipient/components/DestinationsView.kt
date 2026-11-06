package com.gemwallet.android.features.transfer.presents.recipient.components

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.transfer.viewmodels.recipient.models.QrScanField
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.fields.AddressChainField
import com.gemwallet.android.ui.components.fields.MemoTextField
import uniffi.gemstone.GemNameIndicator

fun LazyListScope.destinationView(
    hasMemo: Boolean,
    addressLabel: String,
    address: String,
    addressError: String,
    nameResolveIndicator: GemNameIndicator?,
    memo: String,
    onAddress: (String) -> Unit,
    onMemo: (String) -> Unit,
    onQrScan: (QrScanField) -> Unit,
    onSubmitAddress: () -> Unit,
) {
    item {
        Column {
            AddressChainField(
                value = address,
                label = addressLabel,
                indicator = nameResolveIndicator,
                error = addressError,
                onValueChange = onAddress,
                modifier = Modifier.testTag("addressInputField"),
                onQrScanner = { onQrScan(QrScanField.Address) },
                onSubmit = onSubmitAddress,
            )
            if (hasMemo) {
                MemoTextField(
                    value = memo,
                    label = stringResource(id = R.string.transfer_memo),
                    onValueChange = onMemo,
                    modifier = Modifier.testTag("memoInputField"),
                    onQrScanner = { onQrScan(QrScanField.Memo) },
                )
            }
        }
    }
}
