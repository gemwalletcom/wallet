package com.gemwallet.android.ui.components.list_item.property

import androidx.annotation.StringRes
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ContentCopy
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalClipboard
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.clipboard.setPlainText
import com.gemwallet.android.ui.components.list_item.DropDownContextItem
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.open
import com.wallet.core.primitives.BlockExplorerLink

@Composable
fun AddressPropertyItem(
    @StringRes title: Int,
    displayText: String,
    copyValue: String = displayText,
    explorerLink: BlockExplorerLink? = null,
    listPosition: ListPosition = ListPosition.Middle,
) {
    var isExpanded by remember { mutableStateOf(false) }
    val clipboardManager = LocalClipboard.current.nativeClipboard
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current

    DropDownContextItem(
        isExpanded = isExpanded,
        onDismiss = { isExpanded = false },
        onLongClick = { isExpanded = true },
        onClick = { explorerLink?.let { uriHandler.open(context, it.link) } },
        content = { modifier ->
            PropertyItem(
                modifier = modifier,
                title = { PropertyTitleText(title) },
                data = {
                    PropertyDataText(
                        text = displayText,
                        badge = explorerLink?.let { { DataBadgeChevron() } },
                    )
                },
                listPosition = listPosition,
            )
        },
        menuItems = {
            DropdownMenuItem(
                text = { Text(text = stringResource(id = R.string.wallet_copy_address)) },
                trailingIcon = { Icon(Icons.Default.ContentCopy, contentDescription = null) },
                onClick = {
                    isExpanded = false
                    clipboardManager.setPlainText(context, copyValue)
                },
            )
            if (explorerLink != null) {
                DropdownMenuItem(
                    text = { Text(text = stringResource(id = R.string.transaction_view_on, explorerLink.name)) },
                    trailingIcon = { DataBadgeChevron() },
                    onClick = {
                        isExpanded = false
                        uriHandler.open(context, explorerLink.link)
                    },
                )
            }
        },
    )
}
