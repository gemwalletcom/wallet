package com.gemwallet.android.features.settings.presents.chain_settings

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ActionIcon
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.SelectionCheckmark
import com.gemwallet.android.ui.components.list_item.SwipeableItemWithActions
import com.gemwallet.android.ui.components.list_item.listItemModel
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.WalletTheme
import com.gemwallet.android.ui.theme.paddingSmall
import uniffi.gemstone.GemLatencyStatus
import uniffi.gemstone.GemNodeRow
import uniffi.gemstone.GemNodeRowTitle
import uniffi.gemstone.GemNodeSelection
import uniffi.gemstone.GemNodeSubtitle

@Composable
internal fun ChainNodeItem(row: GemNodeRow, listPosition: ListPosition, isDeleteRevealed: Boolean, onDeleteReveal: () -> Unit, onDeleteCollapse: () -> Unit, onSelect: (String) -> Unit, onDelete: (() -> Unit)?) {
    val context = LocalContext.current
    val model = row.latencyStatus.listItemModel(context, row.title.string(context), row.subtitle.text(context))
    val content: @Composable (ListPosition) -> Unit = { position ->
        ListItem(
            model = model,
            listPosition = position,
            modifier = Modifier.clickable(onClick = { onSelect(row.node.url) }),
            accessory = if (row.node.isSelected) {
                { SelectionCheckmark(modifier = Modifier.padding(end = paddingSmall)) }
            } else {
                null
            },
        )
    }

    if (onDelete == null) {
        content(listPosition)
        return
    }

    SwipeableItemWithActions(
        isRevealed = isDeleteRevealed,
        actions = {
            ActionIcon(
                onClick = onDelete,
                backgroundColor = MaterialTheme.colorScheme.error,
                icon = AppIcons.Delete,
                contentDescription = stringResource(R.string.common_delete),
            )
        },
        listPosition = listPosition,
        onExpanded = onDeleteReveal,
        onCollapsed = onDeleteCollapse,
        content = content,
    )
}

@Preview
@Composable
fun ChainNodeItemPreview() {
    WalletTheme {
        ChainNodeItem(
            row = GemNodeRow(
                node = GemNodeSelection(url = "https://some.url.eth", host = "some.url.eth", isSelected = true, gemNodeFlag = null),
                title = GemNodeRowTitle.Host("some.url.eth"),
                subtitle = GemNodeSubtitle.LatestBlock(null),
                latencyStatus = GemLatencyStatus.Loading,
                canDelete = true,
            ),
            listPosition = ListPosition.Middle,
            isDeleteRevealed = false,
            onDeleteReveal = {},
            onDeleteCollapse = {},
            onSelect = {},
            onDelete = {},
        )
    }
}
