package com.gemwallet.android.features.settings.networks.presents

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.gemwallet.android.features.settings.networks.viewmodels.models.NodeRowUIModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ActionIcon
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.ListItemTextStyle
import com.gemwallet.android.ui.components.list_item.SelectionCheckmark
import com.gemwallet.android.ui.components.list_item.SwipeableItemWithActions
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.WalletTheme
import com.gemwallet.android.ui.theme.paddingSmall
import uniffi.gemstone.GemLatencyStatus
import uniffi.gemstone.GemNodeRow
import uniffi.gemstone.GemNodeRowTitle
import uniffi.gemstone.GemNodeSelection
import uniffi.gemstone.GemNodeSubtitle

@Composable
internal fun NodeItem(model: NodeRowUIModel, listPosition: ListPosition, isDeleteRevealed: Boolean, onDeleteReveal: () -> Unit, onDeleteCollapse: () -> Unit, onSelect: (String) -> Unit, onDelete: (() -> Unit)?) {
    val content: @Composable (ListPosition) -> Unit = { position ->
        ListItem(
            model = model.model,
            listPosition = position,
            modifier = Modifier.clickable(onClick = { onSelect(model.row.node.url) }),
            accessory = if (model.row.node.isSelected) {
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
fun NodeItemPreview() {
    WalletTheme {
        NodeItem(
            model = NodeRowUIModel(
                row = GemNodeRow(
                    node = GemNodeSelection(url = "https://some.url.eth", host = "some.url.eth", isSelected = true, gemNodeFlag = null),
                    title = GemNodeRowTitle.Host("some.url.eth"),
                    subtitle = GemNodeSubtitle.LatestBlock(null),
                    latencyStatus = GemLatencyStatus.Loading,
                    canDelete = true,
                ),
                model = ListItemModel(title = "some.url.eth", titleTag = "440 ms", titleTagStyle = ListItemTextStyle.Positive, titleExtra = "Latest block: 123902302938", titleExtraStyle = ListItemTextStyle.Body),
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
