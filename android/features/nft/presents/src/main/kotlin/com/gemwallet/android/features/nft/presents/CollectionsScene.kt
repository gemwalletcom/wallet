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
import com.gemwallet.android.ui.models.target
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall
import uniffi.gemstone.GemEmptyStateAction
import uniffi.gemstone.GemListPhase
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemNftEntry

private val collectibleCellMinSize = 150.dp

@Composable
internal fun CollectionsScene(
    items: List<GemNftEntry>,
    isRefreshing: Boolean,
    unverifiedListItem: ListItemModel?,
    title: String,
    showReceiveAction: Boolean,
    phase: GemListPhase,
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
            when (phase) {
                is GemListPhase.Empty -> LazyColumn(modifier = Modifier.fillMaxSize()) {
                    item {
                        EmptyContentView(
                            state = phase.state,
                            onAction = { action ->
                                when (action) {
                                    GemEmptyStateAction.RECEIVE -> onAction(CollectionsAction.Receive)
                                    GemEmptyStateAction.BUY, GemEmptyStateAction.SWAP, GemEmptyStateAction.ADD_CUSTOM_TOKEN, GemEmptyStateAction.MANAGE_TOKEN_LIST, GemEmptyStateAction.CLEAR_FILTERS -> Unit
                                }
                            },
                            modifier = Modifier.fillParentMaxSize(),
                        )
                    }
                }

                is GemListPhase.Error -> LazyColumn(modifier = Modifier.fillMaxSize()) {
                    item { GemListRowView(row = GemListRow.Error(phase.error), listPosition = ListPosition.Single) }
                }

                GemListPhase.Rows -> Column(modifier = Modifier.fillMaxSize().padding(top = paddingDefault)) {
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
                                    row = item.row,
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
}
