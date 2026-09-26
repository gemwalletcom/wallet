package com.gemwallet.android.ui.components.list_item.property

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
import com.gemwallet.android.ui.components.clipboard.setCopy
import com.gemwallet.android.ui.components.clipboard.setPlainText
import com.gemwallet.android.ui.components.image.ListItemImageView
import com.gemwallet.android.ui.components.list_item.DropDownContextItem
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemImageStyle
import com.gemwallet.android.ui.components.list_item.listItemImage
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.open
import com.wallet.core.primitives.BlockExplorerLink
import uniffi.gemstone.GemAddressRow
import uniffi.gemstone.GemRowMenuItem

@Composable
fun AddressPropertyItem(row: GemAddressRow, listPosition: ListPosition = ListPosition.Middle, onClick: (() -> Unit)? = null) {
    var isExpanded by remember { mutableStateOf(false) }
    var showsAddress by remember { mutableStateOf(false) }
    val context = LocalContext.current
    val clipboardManager = context.clipboardManager()
    val uriHandler = LocalUriHandler.current
    val select = onClick.takeIf { row.isSelectable }
    val image = row.avatar?.listItemImage()

    DropDownContextItem(
        isExpanded = isExpanded,
        onDismiss = { isExpanded = false },
        onLongClick = { isExpanded = row.menu.isNotEmpty() },
        onClick = select ?: { showsAddress = row.shortAddress != null && !showsAddress },
        content = { modifier ->
            PropertyItem(
                modifier = modifier,
                title = { PropertyTitleText(text = row.title.string(context)) },
                data = {
                    PropertyDataText(
                        text = row.shortAddress?.takeIf { showsAddress } ?: row.text.string(context),
                        badge = when {
                            image != null -> {
                                { DataBadgeChevron(select != null) { ListItemImageView(image = image, style = ListItemImageStyle.Glyph) } }
                            }

                            select != null -> {
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
            row.menu.forEach { item ->
                when (item) {
                    is GemRowMenuItem.Copy -> DropdownMenuItem(
                        text = { Text(text = stringResource(id = R.string.common_copy)) },
                        trailingIcon = { Icon(AppIcons.ContentCopy, contentDescription = null) },
                        onClick = {
                            isExpanded = false
                            clipboardManager.setCopy(context, item.copy)
                        },
                    )

                    is GemRowMenuItem.Open -> DropdownMenuItem(
                        text = { Text(text = item.title.string(context)) },
                        trailingIcon = { DataBadgeChevron() },
                        onClick = {
                            isExpanded = false
                            uriHandler.open(context, item.url)
                        },
                    )
                }
            }
        },
    )
}

@Composable
fun AddressPropertyItem(title: String, displayText: String, copyValue: String, image: ListItemImage? = null, explorerLink: BlockExplorerLink? = null, listPosition: ListPosition = ListPosition.Middle, onClick: (() -> Unit)? = null) {
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
                                { DataBadgeChevron(onClick != null || explorerLink != null) { ListItemImageView(image = image, style = ListItemImageStyle.Glyph) } }
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
