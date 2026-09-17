package com.gemwallet.android.ui.components.list_item.property

import androidx.annotation.StringRes
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.clipboard.clipboardManager
import com.gemwallet.android.ui.components.clipboard.setPlainText
import com.gemwallet.android.ui.components.image.ListItemImageView
import com.gemwallet.android.ui.components.list_item.DropDownContextItem
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.open
import com.gemwallet.android.ui.theme.smallIconSize
import com.wallet.core.primitives.BlockExplorerLink

@Composable
fun AddressPropertyItem(
    @StringRes title: Int,
    displayText: String,
    copyValue: String,
    image: ListItemImage? = null,
    explorerLink: BlockExplorerLink? = null,
    listPosition: ListPosition = ListPosition.Middle,
    onClick: (() -> Unit)? = null,
) {
    AddressPropertyItem(
        title = stringResource(title),
        displayText = displayText,
        copyValue = copyValue,
        image = image,
        explorerLink = explorerLink,
        listPosition = listPosition,
        onClick = onClick,
    )
}

@Composable
fun AddressPropertyItem(
    title: String,
    displayText: String,
    copyValue: String,
    image: ListItemImage? = null,
    explorerLink: BlockExplorerLink? = null,
    listPosition: ListPosition = ListPosition.Middle,
    onClick: (() -> Unit)? = null,
) {
    var isExpanded by remember { mutableStateOf(false) }
    val clipboardManager = LocalContext.current.clipboardManager()
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current

    DropDownContextItem(
        isExpanded = isExpanded,
        onDismiss = { isExpanded = false },
        onLongClick = { isExpanded = true },
        onClick = onClick ?: { explorerLink?.let { link -> uriHandler.open(context, link.link) } ?: Unit },
        content = { modifier ->
            PropertyItem(
                modifier = modifier,
                title = { PropertyTitleText(text = title) },
                data = {
                    PropertyDataText(
                        text = displayText,
                        badge = when {
                            image != null -> {
                                { DataBadgeChevron(onClick != null || explorerLink != null) { ListItemImageView(image = image, size = smallIconSize) } }
                            }
                            onClick != null || explorerLink != null -> {
                                { DataBadgeChevron() }
                            }
                            else -> null
                        },
                    )
                },
                listPosition = listPosition,
            )
        },
        menuItems = {
            DropdownMenuItem(
                text = { Text(text = stringResource(id = R.string.wallet_copy_address)) },
                trailingIcon = { Icon(AppIcons.ContentCopy, contentDescription = null) },
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
