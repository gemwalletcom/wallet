package com.gemwallet.android.features.activities.presents.details.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ContentCopy
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalClipboard
import androidx.compose.ui.platform.LocalContext
import com.gemwallet.android.domains.transaction.values.TransactionDetailsValue
import com.gemwallet.android.ext.AddressFormatter
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.clipboard.setPlainText
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.paddingSmall

@Composable
fun ValidatorPropertyItem(property: TransactionDetailsValue.Validator, listPosition: ListPosition) {
    val context = LocalContext.current
    val clipboardManager = LocalClipboard.current.nativeClipboard

    PropertyItem(
        title = { PropertyTitleText(R.string.stake_validator) },
        data = {
            PropertyDataText(
                text = AddressFormatter(property.address).value(),
                modifier = Modifier.clickable { clipboardManager.setPlainText(context, property.address) },
                badge = {
                    Icon(
                        modifier = Modifier.padding(start = paddingSmall),
                        imageVector = Icons.Default.ContentCopy,
                        tint = MaterialTheme.colorScheme.secondary,
                        contentDescription = null,
                    )
                }
            )
        },
        listPosition = listPosition,
    )
}
