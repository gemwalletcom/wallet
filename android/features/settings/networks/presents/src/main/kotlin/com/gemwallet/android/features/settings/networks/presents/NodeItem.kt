package com.gemwallet.android.features.settings.networks.presents

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
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
import com.gemwallet.android.features.settings.networks.presents.localization.string
import com.gemwallet.android.features.settings.networks.presents.localization.stringRes
import uniffi.gemstone.GemNodeRow
import uniffi.gemstone.GemNodeSelection
import uniffi.gemstone.GemNodeStatusState
import uniffi.gemstone.GemNodeSubtitle
import uniffi.gemstone.GemNodeRowTitle
import uniffi.gemstone.Latency
import uniffi.gemstone.LatencyType

@Composable
internal fun NodeItem(
    model: GemNodeRow,
    listPosition: ListPosition,
    isDeleteRevealed: Boolean,
    onDeleteReveal: () -> Unit,
    onDeleteCollapse: () -> Unit,
    onSelect: (String) -> Unit,
    onDelete: (() -> Unit)?,
) {
    val content: @Composable (ListPosition) -> Unit = { position ->
        ListItem(
            modifier = Modifier.clickable(onClick = { onSelect(model.node.url) }),
            title = {
                ListItemTitleText(
                    text = model.title.string(),
                    titleBadge = {
                        LatencyStatusBadge(status = model.latencyStatus)
                    },
                )
            },
            subtitle = {
                ListItemSupportText(
                    text = model.subtitleText(),
                    color = MaterialTheme.colorScheme.onSurface,
                )
            },
            listPosition = position,
            trailing = if (model.node.isSelected) {
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
private fun GemNodeRow.subtitleText(): String {
    val value = when (val subtitle = subtitle) {
        is GemNodeSubtitle.LatestBlock -> subtitle.value
    }

    return "${stringResource(subtitle.stringRes())}: $value"
}

@Preview
@Composable
fun NodeItemPreview() {
    WalletTheme {
        val status = GemNodeStatusState.Result(
            latestBlockNumber = 123902302938UL,
            latency = Latency(LatencyType.FAST, 440.0),
        )
        NodeItem(
            model = GemNodeRow(
                node = GemNodeSelection(
                    url = "https://some.url.eth",
                    host = "some.url.eth",
                    isSelected = true,
                    gemNodeFlag = null,
                ),
                title = GemNodeRowTitle.Host("some.url.eth"),
                subtitle = status.subtitle(),
                latencyStatus = status.latencyStatus(),
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
