package com.gemwallet.android.features.nft.presents

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyGridState
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.grid.rememberLazyGridState
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import com.gemwallet.android.features.nft.presents.components.NftItem
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.empty.EmptyContentType
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.screen.PullToRefreshBox
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.NftItemTarget
import com.gemwallet.android.ui.models.NftItemUIModel
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall
import uniffi.gemstone.GemEmptyStateAction
import uniffi.gemstone.GemEmptyStateKind
import uniffi.gemstone.GemListRow

private val collectibleCellMinSize = 150.dp

@Composable
internal fun CollectionsScene(
    items: List<NftItemUIModel>,
    isRefreshing: Boolean,
    errorRow: GemListRow?,
    unverifiedListItem: ListItemModel?,
    title: String,
    showReceiveAction: Boolean,
    listState: LazyGridState = rememberLazyGridState(),
    snackbar: SnackbarHostState? = null,
    onAction: (CollectionsAction) -> Unit,
) {
    Scene(
        title = title,
        actions = {
            if (showReceiveAction) {
                IconButton(onClick = { onAction(CollectionsAction.Receive) }) {
                    Icon(
                        imageVector = AppIcons.Add,
                        contentDescription = stringResource(R.string.wallet_receive),
                    )
                }
            }
        },
        onClose = { onAction(CollectionsAction.Close) },
        snackbar = snackbar,
    ) {
        PullToRefreshBox(
            modifier = Modifier.fillMaxSize(),
            isRefreshing = isRefreshing,
            onRefresh = { onAction(CollectionsAction.Refresh) },
        ) {
            if (items.isEmpty() && unverifiedListItem == null) {
                LazyColumn(modifier = Modifier.fillMaxSize()) {
                    item {
                        if (errorRow == null) {
                            EmptyContentView(
                                type = nftEmptyContentType(
                                    showReceiveAction = showReceiveAction,
                                    onAction = onAction,
                                ),
                                modifier = Modifier.fillParentMaxSize(),
                            )
                        } else {
                            GemListRowView(row = errorRow, listPosition = ListPosition.Single)
                        }
                    }
                }
                return@PullToRefreshBox
            }

            Column(modifier = Modifier.fillMaxSize().padding(top = paddingDefault)) {
                if (items.isNotEmpty()) {
                    LazyVerticalGrid(
                        modifier = Modifier
                            .weight(1f)
                            .fillMaxWidth(),
                        columns = GridCells.Adaptive(minSize = collectibleCellMinSize),
                        state = listState,
                        contentPadding = PaddingValues(
                            start = paddingSmall,
                            end = paddingSmall,
                            bottom = paddingDefault,
                        ),
                    ) {
                        items(items) { item ->
                            NftItem(
                                model = item,
                                onClick = {
                                    when (val target = item.target) {
                                        is NftItemTarget.Collection -> onAction(CollectionsAction.OpenCollection(target.id))
                                        is NftItemTarget.Asset -> onAction(CollectionsAction.OpenAsset(target.id))
                                    }
                                },
                            )
                        }
                    }
                }
                unverifiedListItem?.let { model ->
                    ListItem(
                        model = model,
                        listPosition = ListPosition.Single,
                        modifier = Modifier.clickable { onAction(CollectionsAction.OpenUnverified) },
                        accessory = { DataBadgeChevron() },
                    )
                }
            }
        }
    }
}

private fun nftEmptyContentType(showReceiveAction: Boolean, onAction: (CollectionsAction) -> Unit): EmptyContentType {
    val onReceive: (() -> Unit)? = if (showReceiveAction) {
        { onAction(CollectionsAction.Receive) }
    } else {
        null
    }

    return EmptyContentType(GemEmptyStateKind.NFTS, actions = mapOf(GemEmptyStateAction.RECEIVE to onReceive))
}
