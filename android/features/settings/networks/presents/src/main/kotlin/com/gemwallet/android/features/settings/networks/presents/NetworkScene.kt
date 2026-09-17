@file:OptIn(ExperimentalMaterial3Api::class)

package com.gemwallet.android.features.settings.networks.presents

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.slideIn
import androidx.compose.animation.slideOut
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.IntOffset
import com.gemwallet.android.ext.networkName
import com.gemwallet.android.features.settings.networks.viewmodels.models.ExplorerRowUIModel
import com.gemwallet.android.features.settings.networks.viewmodels.models.NetworkSectionUIModel
import com.gemwallet.android.features.settings.networks.viewmodels.models.NetworksUIState
import com.gemwallet.android.features.settings.networks.viewmodels.models.NodeRowUIModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.SelectionCheckmark
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.PullToRefreshBox
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.paddingSmall

@Composable
internal fun NetworkScene(
    state: NetworksUIState,
    snackbar: SnackbarHostState? = null,
    onAction: (NetworkAction) -> Unit,
) {
    val chain = state.chain ?: return
    var isShowAddSource by remember { mutableStateOf(false) }
    var revealedNodeId by remember { mutableStateOf<String?>(null) }
    var nodeDelete by remember { mutableStateOf<NodeRowUIModel?>(null) }

    Scene(
        title = chain.networkName(),
        snackbar = snackbar,
        actions = {
            if (state.availableAddNode) {
                IconButton(onClick = { isShowAddSource = true }) {
                    Icon(imageVector = AppIcons.Add, contentDescription = "")
                }
            }
        },
        onClose = { onAction(NetworkAction.Cancel) },
    ) {
        PullToRefreshBox(
            isRefreshing = false,
            onRefresh = { onAction(NetworkAction.Refresh) },
        ) {
            LazyColumn(modifier = Modifier.fillMaxSize()) {
                state.sections.forEach { section ->
                    item { SubheaderItem(section.title) }
                    when (section) {
                        is NetworkSectionUIModel.Nodes -> itemsIndexed(section.rows, key = { _, item -> item.url }) { index, node ->
                            NodeItem(
                                model = node,
                                listPosition = ListPosition.getPosition(index, section.rows.size),
                                isDeleteRevealed = revealedNodeId == node.url,
                                onDeleteReveal = { revealedNodeId = node.url },
                                onDeleteCollapse = {
                                    if (revealedNodeId == node.url) {
                                        revealedNodeId = null
                                    }
                                },
                                onSelect = { onAction(NetworkAction.SelectNode(it)) },
                                onDelete = if (node.canDelete) {
                                    {
                                        revealedNodeId = null
                                        nodeDelete = node
                                    }
                                } else {
                                    null
                                },
                            )
                        }
                        is NetworkSectionUIModel.Explorers -> itemsPositioned(section.rows) { position, item ->
                            BlockExplorerItem(item, position) { onAction(NetworkAction.SelectBlockExplorer(it)) }
                        }
                    }
                }
            }
        }
    }

    AnimatedVisibility(
        visible = isShowAddSource,
        label = "",
        enter = slideIn { IntOffset(it.width, 0) },
        exit = slideOut { IntOffset(it.width, 0) },
    ) {
        AddNodeScene(
            chain = chain,
            onCancel = {
                isShowAddSource = false
                onAction(NetworkAction.Refresh)
            },
        )
    }

    nodeDelete?.let { pendingNode ->
        ConfirmNodeDeleteDialog(
            nodeName = pendingNode.host,
            onConfirm = {
                onAction(NetworkAction.DeleteNode(pendingNode.url))
                nodeDelete = null
            },
            onDismiss = { nodeDelete = null },
        )
    }
}

@Composable
private fun BlockExplorerItem(
    explorer: ExplorerRowUIModel,
    listPosition: ListPosition,
    onSelect: (String) -> Unit,
) {
    ListItem(
        model = explorer.model,
        listPosition = listPosition,
        modifier = Modifier.clickable { onSelect(explorer.name) },
        accessory = if (explorer.isSelected) {
            { SelectionCheckmark(modifier = Modifier.padding(end = paddingSmall)) }
        } else {
            null
        },
    )
}

@Composable
private fun ConfirmNodeDeleteDialog(
    nodeName: String,
    onConfirm: () -> Unit,
    onDismiss: () -> Unit,
) {
    AlertDialog(
        onDismissRequest = onDismiss,
        containerColor = MaterialTheme.colorScheme.background,
        title = {
            Text(stringResource(R.string.common_warning))
        },
        text = {
            Text(
                text = stringResource(R.string.common_delete_confirmation, nodeName),
                style = MaterialTheme.typography.bodyLarge,
            )
        },
        confirmButton = {
            TextButton(
                colors = ButtonDefaults.textButtonColors().copy(contentColor = MaterialTheme.colorScheme.error),
                onClick = onConfirm,
            ) {
                Text(text = stringResource(id = R.string.common_delete))
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text(stringResource(R.string.common_cancel))
            }
        }
    )
}
