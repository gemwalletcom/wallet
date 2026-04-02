package com.gemwallet.android.features.settings.networks.presents

import android.icu.text.DecimalFormat
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
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
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator14
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.Spacer6
import com.gemwallet.android.ui.theme.WalletTheme
import com.gemwallet.android.ui.theme.alpha10
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.space2
import com.gemwallet.android.ui.theme.space6
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
                    titleBadge = { NodeItemStatus(model.statusState) }
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
private fun NodeItemStatus(statusState: NodeStatusState) {
    if (statusState is NodeStatusState.Loading) {
        Spacer6()
        CircularProgressIndicator14()
        return
    }

    val color = statusState.statusColor()
    Row(
        Modifier
            .padding(start = paddingHalfSmall)
            .background(color = color.copy(alpha = alpha10), shape = RoundedCornerShape(space6)),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            modifier = Modifier.padding(
                start = paddingHalfSmall,
                top = space2,
                end = paddingHalfSmall,
                bottom = space2,
            ),
            text = statusState.statusText(),
            color = color,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            style = MaterialTheme.typography.labelMedium,
        )
    }
}

@Composable
private fun NodeStatusState.statusText(): String = when (this) {
    NodeStatusState.Error -> stringResource(R.string.errors_error)
    is NodeStatusState.Loading -> ""
    is NodeStatusState.Result -> stringResource(R.string.common_latency_in_ms, latency.toLong())
}

@Composable
private fun NodeStatusState.statusColor(): Color = when (this) {
    NodeStatusState.Error -> MaterialTheme.colorScheme.error
    is NodeStatusState.Loading -> Color.Transparent
    is NodeStatusState.Result -> {
        when {
            latency < 1024UL -> MaterialTheme.colorScheme.tertiary
            latency < 2048UL -> Color(0xffff9314)
            else -> MaterialTheme.colorScheme.error
        }
    }
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
