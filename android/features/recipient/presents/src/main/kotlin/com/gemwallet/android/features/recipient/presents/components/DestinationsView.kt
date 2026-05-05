package com.gemwallet.android.features.recipient.presents.components

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.features.recipient.viewmodel.models.QrScanField
import com.gemwallet.android.features.recipient.viewmodel.models.RecipientError
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.ui.R
import com.wallet.core.primitives.NameRecord

fun LazyListScope.destinationView(
    asset: AssetInfo,
    hasMemo: Boolean,
    address: String,
    addressError: RecipientError,
    memo: String,
    memoError: RecipientError,
    onAddress: (String, NameRecord?) -> Unit,
    onMemo: (String) -> Unit,
    onQrScan: (QrScanField) -> Unit,
) {
    item {
        Column {
            AddressChainField(
                chain = asset.asset.chain,
                value = address,
                label = stringResource(id = R.string.transfer_recipient_address_field),
                error = recipientErrorString(error = addressError),
                onValueChange = onAddress,
                onQrScanner = { onQrScan(QrScanField.Address) }
            )
            if (hasMemo) {
                MemoTextField(
                    value = memo,
                    label = stringResource(id = R.string.transfer_memo),
                    onValueChange = onMemo,
                    error = memoError,
                    onQrScanner = { onQrScan(QrScanField.Memo) },
                )
            }
        }
    }
}
