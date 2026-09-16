package com.gemwallet.android.features.settings.networks.presents

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.res.stringResource
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
import com.gemwallet.android.features.settings.networks.viewmodels.models.LatencyTone
import com.gemwallet.android.features.settings.networks.viewmodels.models.LatencyUIModel
import com.gemwallet.android.features.settings.networks.viewmodels.models.NodeRowUIModel

@Composable
internal fun NodeItem(
    model: NodeRowUIModel,
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
                    text = model.title,
                    titleBadge = {
                        LatencyStatusBadge(latency = model.latency)
                    },
                )
            },
            subtitle = {
                ListItemSupportText(
                    text = model.subtitle,
                    color = MaterialTheme.colorScheme.onSurface,
                )
            },
            listPosition = position,
            trailing = if (model.isSelected) {
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

@Preview
@Composable
fun NodeItemPreview() {
    WalletTheme {
        NodeItem(
            model = NodeRowUIModel(
                url = "https://some.url.eth",
                host = "some.url.eth",
                title = "some.url.eth",
                subtitle = "Latest block: 123902302938",
                latency = LatencyUIModel(text = "440 ms", tone = LatencyTone.Fast),
                isSelected = true,
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
