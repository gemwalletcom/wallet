package com.gemwallet.android.features.settings.networks.presents

import android.icu.text.DecimalFormat
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.gemwallet.android.features.settings.networks.viewmodels.models.NodeRowUiModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ActionIcon
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemSupportText
import com.gemwallet.android.ui.components.list_item.ListItemTitleText
import com.gemwallet.android.ui.components.list_item.SelectionCheckmark
import com.gemwallet.android.ui.components.list_item.SwipeableItemWithActions
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.WalletTheme
import com.gemwallet.android.ui.theme.paddingSmall
import uniffi.gemstone.GemNodeRowTitle
import uniffi.gemstone.GemNodeSelection
import uniffi.gemstone.GemNodeStatusState
import uniffi.gemstone.Latency
import uniffi.gemstone.LatencyType
import com.gemwallet.android.ui.theme.Placeholder

@Composable
internal fun NodeItem(
    model: NodeRowUiModel,
    listPosition: ListPosition,
    isDeleteRevealed: Boolean,
    onDeleteReveal: () -> Unit,
    onDeleteCollapse: () -> Unit,
    onSelect: (String) -> Unit,
    onDelete: (() -> Unit)?,
) {
    val content: @Composable (ListPosition) -> Unit = { position ->
        ListItem(
            modifier = Modifier.clickable(onClick = { onSelect(model.url) }),
            title = {
                ListItemTitleText(
                    text = model.title(),
                    titleBadge = {
                        LatencyStatusBadge(status = model.statusState.latencyStatus())
                    },
                )
            },
            subtitle = {
                ListItemSupportText(
                    text = model.latestBlockText(),
                    color = MaterialTheme.colorScheme.onSurface,
                )
            },
            listPosition = position,
            trailing = if (model.selected) {
                @Composable {
                    SelectionCheckmark(modifier = Modifier.padding(end = paddingSmall))
                }
            } else null
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

@Composable
private fun NodeRowUiModel.title(): String = when (val title = node.title()) {
    is GemNodeRowTitle.Host -> title.host
    is GemNodeRowTitle.GemNode -> "${stringResource(R.string.nodes_gem_wallet_node)} ${title.flag}"
}

@Composable
private fun NodeRowUiModel.latestBlockText(): String {
    val blockValue = statusState.latestBlock()?.let { DecimalFormat.getInstance().format(it.toLong()) } ?: Placeholder.empty

    return "${stringResource(R.string.nodes_import_node_latest_block)}: $blockValue"
}

@Preview
@Composable
fun NodeItemPreview() {
    WalletTheme {
        NodeItem(
            model = NodeRowUiModel(
                node = GemNodeSelection(
                    url = "https://some.url.eth",
                    host = "some.url.eth",
                    isSelected = true,
                    gemNodeFlag = null,
                ),
                canDelete = true,
                statusState = GemNodeStatusState.Result(
                    latestBlockNumber = 123902302938UL,
                    latency = Latency(LatencyType.FAST, 440.0),
                ),
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
