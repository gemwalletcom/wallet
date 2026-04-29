@file:OptIn(ExperimentalMaterial3Api::class)

package com.gemwallet.android.features.settings.networks.presents

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.slideIn
import androidx.compose.animation.slideOut
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.material3.pulltorefresh.PullToRefreshDefaults.Indicator
import androidx.compose.material3.pulltorefresh.rememberPullToRefreshState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.IntOffset
import com.gemwallet.android.ext.asset
import com.gemwallet.android.features.settings.networks.viewmodels.models.NetworksUIState
import com.gemwallet.android.features.settings.networks.viewmodels.models.NodeRowUiModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.SelectionCheckmark
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.paddingSmall
import com.wallet.core.primitives.Node

@Composable
fun NetworkScene(
    state: NetworksUIState,
    onRefresh: () -> Unit,
    onSelectNode: (Node) -> Unit,
    onDeleteNode: (Node) -> Unit,
    onSelectBlockExplorer: (String) -> Unit,
    onCancel: () -> Unit,
) {
    val chain = state.chain ?: return
    var isShowAddSource by remember { mutableStateOf(false) }
    var revealedNodeId by remember { mutableStateOf<String?>(null) }
    var nodeDelete by remember { mutableStateOf<NodeRowUiModel?>(null) }

    Scene(
        title = chain.asset().name,
        actions = {
            if (state.availableAddNode) {
                IconButton(onClick = { isShowAddSource = true }) {
                    Icon(imageVector = Icons.Default.Add, contentDescription = "")
                }
            }
        },
        onClose = onCancel,
    ) {
        val pullToRefreshState = rememberPullToRefreshState()
        PullToRefreshBox(
            isRefreshing = state.isRefreshing,
            onRefresh = onRefresh,
            state = pullToRefreshState,
            enabled = true,
            indicator = {
                if (pullToRefreshState.distanceFraction > 0f || state.isRefreshing) {
                    Indicator(
                        modifier = Modifier.align(Alignment.TopCenter),
                        isRefreshing = state.isRefreshing,
                        state = pullToRefreshState,
                        containerColor = MaterialTheme.colorScheme.background,
                    )
                }
            },
        ) {
            LazyColumn {
                item {
                    SubheaderItem(R.string.settings_networks_source)
                }

                val size = state.nodeRows.size
                itemsIndexed(state.nodeRows, key = { _, item -> item.id }) { index, node ->
                    NodeItem(
                        model = node,
                        listPosition = ListPosition.getPosition(index, size),
                        isDeleteRevealed = revealedNodeId == node.id,
                        onDeleteReveal = { revealedNodeId = node.id },
                        onDeleteCollapse = {
                            if (revealedNodeId == node.id) {
                                revealedNodeId = null
                            }
                        },
                        onSelect = onSelectNode,
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

                item {
                    SubheaderItem(R.string.settings_networks_explorer)
                }
                itemsPositioned(state.blockExplorers) { position, item ->
                    BlockExplorerItem(state.currentExplorer, item, position, onSelectBlockExplorer)
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
                onRefresh()
            },
        )
    }

    nodeDelete?.let { pendingNode ->
        ConfirmNodeDeleteDialog(
            nodeName = pendingNode.host,
            onConfirm = {
                onDeleteNode(pendingNode.node)
                nodeDelete = null
            },
            onDismiss = { nodeDelete = null },
        )
    }
}

@Composable
private fun BlockExplorerItem(
    current: String?,
    explorerName: String,
    listPosition: ListPosition,
    onSelect: (String) -> Unit,
) {
    PropertyItem(
        modifier = Modifier.clickable { onSelect(explorerName) },
        title = {
            PropertyTitleText(text = explorerName)
        },
        data = if (explorerName == current) {
            {
                SelectionCheckmark(modifier = Modifier.padding(end = paddingSmall))
            }
        } else null,
        listPosition = listPosition,
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
