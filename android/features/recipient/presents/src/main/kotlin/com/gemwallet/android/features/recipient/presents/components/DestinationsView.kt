package com.gemwallet.android.features.recipient.presents.components

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.recipient.viewmodel.models.QrScanField
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.fields.AddressChainField
import com.gemwallet.android.ui.components.fields.MemoTextField
import com.gemwallet.android.ui.components.fields.NameResolveIndicatorUIModel

fun LazyListScope.destinationView(
    hasMemo: Boolean,
    address: String,
    addressError: String,
    nameResolveIndicator: NameResolveIndicatorUIModel?,
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
                label = stringResource(id = R.string.transfer_recipient_address_field),
                indicator = nameResolveIndicator,
                error = addressError,
                onValueChange = onAddress,
                onQrScanner = { onQrScan(QrScanField.Address) },
                onSubmit = onSubmitAddress,
            )
            if (hasMemo) {
                MemoTextField(
                    value = memo,
                    label = stringResource(id = R.string.transfer_memo),
                    onValueChange = onMemo,
                    onQrScanner = { onQrScan(QrScanField.Memo) },
                )
            }
        }
    }
}
