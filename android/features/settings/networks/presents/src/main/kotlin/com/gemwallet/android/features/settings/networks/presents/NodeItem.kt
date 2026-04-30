package com.gemwallet.android.features.settings.networks.presents

import android.icu.text.DecimalFormat
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.gemwallet.android.features.settings.networks.viewmodels.models.NodeRowUiModel
import com.gemwallet.android.features.settings.networks.viewmodels.models.NodeStatusState
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ActionIcon
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemSupportText
import com.gemwallet.android.ui.components.list_item.ListItemTitleText
import com.gemwallet.android.ui.components.list_item.SelectionCheckmark
import com.gemwallet.android.ui.components.list_item.SwipeableItemWithActions
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.WalletTheme
import com.gemwallet.android.ui.theme.paddingSmall
import com.wallet.core.primitives.Node
import com.wallet.core.primitives.NodeState

@Composable
internal fun NodeItem(
    model: NodeRowUiModel,
    listPosition: ListPosition,
    isDeleteRevealed: Boolean,
    onDeleteReveal: () -> Unit,
    onDeleteCollapse: () -> Unit,
    onSelect: (Node) -> Unit,
    onDelete: (() -> Unit)?,
) {
    val content: @Composable (ListPosition) -> Unit = { position ->
        ListItem(
            modifier = Modifier.clickable(onClick = { onSelect(model.node) }),
            title = {
                ListItemTitleText(
                    text = model.title(),
                    titleBadge = {
                        LatencyStatusBadge(
                            latency = model.statusState.latency,
                            isLoading = model.statusState is NodeStatusState.Loading,
                        )
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
                icon = Icons.Default.Delete,
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
private fun NodeRowUiModel.title(): String {
    return gemNodeFlag?.let { "${stringResource(R.string.nodes_gem_wallet_node)} $it" } ?: host
}

@Composable
private fun NodeRowUiModel.latestBlockText(): String {
    val blockValue = when (val currentState = statusState) {
        NodeStatusState.Error,
        NodeStatusState.Loading -> "-"
        is NodeStatusState.Result -> currentState.latestBlock
            .takeIf { it > 0UL }
            ?.let { DecimalFormat.getInstance().format(it.toLong()) }
            ?: "-"
    }

    return "${stringResource(R.string.nodes_import_node_latest_block)}: $blockValue"
}

private val NodeStatusState.latency: ULong?
    get() = (this as? NodeStatusState.Result)?.latency

@Preview
@Composable
fun NodeItemPreview() {
    WalletTheme {
        NodeItem(
            model = NodeRowUiModel(
                node = Node(
                    url = "https://some.url.eth",
                    status = NodeState.Active,
                    priority = 0,
                ),
                host = "some.url.eth",
                selected = true,
                canDelete = true,
                statusState = NodeStatusState.Result(
                    latestBlock = 123902302938UL,
                    latency = 440UL,
                    chainId = "ethereum",
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
