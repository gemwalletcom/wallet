package com.gemwallet.android.features.activities.presents.details.components

import androidx.compose.foundation.clickable
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ContentCopy
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalClipboard
import androidx.compose.ui.platform.LocalContext
import androidx.compose.foundation.layout.padding
import com.gemwallet.android.domains.transaction.values.TransactionDetailsValue
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.clipboard.setPlainText
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.paddingSmall

@Composable
fun DestinationPropertyItem(property: TransactionDetailsValue.Destination, listPosition: ListPosition) {
    val context = LocalContext.current
    val clipboardManager = LocalClipboard.current.nativeClipboard
    val title = when (property) {
        is TransactionDetailsValue.Destination.Recipient -> R.string.transaction_recipient
        is TransactionDetailsValue.Destination.Sender -> R.string.transaction_sender
        is TransactionDetailsValue.Destination.Provider -> R.string.common_provider
    }
    val isCopied = when (property) {
        is TransactionDetailsValue.Destination.Provider -> false
        else -> true
    }

    PropertyItem(
        title = { PropertyTitleText(title) },
        data = {
            PropertyDataText(
                text = property.data,
                modifier = Modifier
                    .clickable(enabled = isCopied) { clipboardManager.setPlainText(context, property.data) },
                badge = if (isCopied) {
                    {
                        Icon(
                            modifier = Modifier.padding(start = paddingSmall),
                            imageVector = Icons.Default.ContentCopy,
                            tint = MaterialTheme.colorScheme.secondary,
                            contentDescription = null,
                        )
                    }
                } else {
                    null
                }
            )
        },
        listPosition = listPosition,
    )
}
