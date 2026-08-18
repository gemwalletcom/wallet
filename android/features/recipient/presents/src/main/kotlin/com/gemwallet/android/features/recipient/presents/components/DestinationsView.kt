package com.gemwallet.android.features.recipient.presents.components

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.recipient.viewmodel.models.QrScanField
import com.gemwallet.android.features.recipient.viewmodel.models.RecipientError
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.fields.AddressChainField
import com.gemwallet.android.ui.components.fields.MemoTextField
import com.gemwallet.android.ui.models.name.NameRecordState

fun LazyListScope.destinationView(
    hasMemo: Boolean,
    assetName: String,
    address: String,
    addressError: Boolean,
    nameResolveState: NameRecordState,
    memo: String,
    memoError: RecipientError,
    onAddress: (String) -> Unit,
    onMemo: (String) -> Unit,
    onQrScan: (QrScanField) -> Unit,
) {
    item {
        Column {
            AddressChainField(
                value = address,
                label = stringResource(id = R.string.transfer_recipient_address_field),
                state = nameResolveState,
                error = if (addressError) stringResource(R.string.errors_invalid_asset_address, assetName) else "",
                onValueChange = onAddress,
                onQrScanner = { onQrScan(QrScanField.Address) }
            )
            if (hasMemo) {
                MemoTextField(
                    value = memo,
                    label = stringResource(id = R.string.transfer_memo),
                    onValueChange = onMemo,
                    error = recipientErrorString(error = memoError),
                    onQrScanner = { onQrScan(QrScanField.Memo) },
                )
            }
        }
    }
}
