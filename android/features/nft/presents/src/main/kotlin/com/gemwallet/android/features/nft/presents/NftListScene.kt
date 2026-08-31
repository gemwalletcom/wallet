package com.gemwallet.android.features.nft.presents

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyGridState
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.grid.rememberLazyGridState
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.nft.presents.components.NFTItem
import com.gemwallet.android.features.nft.viewmodels.NftListMode
import com.gemwallet.android.features.nft.viewmodels.NftListViewModels
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.empty.EmptyContentType
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.list_item.LinkItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.screen.PullToRefreshBox
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.NftItemUIModel
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ui.models.actions.NftAssetIdAction
import com.gemwallet.android.ui.models.actions.NftCollectionIdAction
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall

@Composable
fun NftListNavScreen(
    cancelAction: CancelAction,
    collectionAction: NftCollectionIdAction,
    assetAction: NftAssetIdAction,
    onReceive: () -> Unit,
    onUnverified: () -> Unit,
    listState: LazyGridState = rememberLazyGridState(),
    viewModel: NftListViewModels = hiltViewModel(),
) {
    val items by viewModel.collections.collectAsStateWithLifecycle()
    val isRefreshing by viewModel.isRefreshing.collectAsStateWithLifecycle()
    val unverifiedCount by viewModel.unverifiedCount.collectAsStateWithLifecycle()
    val walletId by viewModel.walletId.collectAsStateWithLifecycle()

    LaunchedEffect(walletId) {
        viewModel.syncIfNeeded()
    }

    NftListScene(
        items = items,
        isRefreshing = isRefreshing,
        unverifiedCount = unverifiedCount,
        mode = viewModel.mode,
        listState = listState,
        onAction = { action ->
            when (action) {
                NftListAction.Refresh -> viewModel.refresh()
                NftListAction.Close -> cancelAction()
                NftListAction.Receive -> onReceive()
                NftListAction.OpenUnverified -> onUnverified()
                is NftListAction.OpenCollection -> collectionAction(action.collectionId)
                is NftListAction.OpenAsset -> assetAction(action.assetId)
            }
        },
    )
}

@Composable
internal fun NftListScene(
    items: List<NftItemUIModel>,
    isRefreshing: Boolean,
    unverifiedCount: Int,
    mode: NftListMode,
    listState: LazyGridState = rememberLazyGridState(),
    onAction: (NftListAction) -> Unit,
) {
    val showReceiveAction = mode != NftListMode.Unverified

    Scene(
        title = when (mode) {
            NftListMode.Collections,
            is NftListMode.Collection -> stringResource(R.string.nft_collections)
            NftListMode.Unverified -> stringResource(R.string.asset_verification_unverified)
        },
        actions = {
            if (showReceiveAction) {
                IconButton(onClick = { onAction(NftListAction.Receive) }) {
                    Icon(
                        imageVector = AppIcons.Add,
                        contentDescription = stringResource(R.string.wallet_receive),
                    )
                }
            }
        },
        onClose = { onAction(NftListAction.Close) },
    ) {
        PullToRefreshBox(
            modifier = Modifier.fillMaxSize(),
            isRefreshing = isRefreshing,
            onRefresh = { onAction(NftListAction.Refresh) },
        ) {
            val showUnverifiedRow = mode == NftListMode.Collections && unverifiedCount > 0

            if (items.isEmpty() && !showUnverifiedRow) {
                LazyColumn(modifier = Modifier.fillMaxSize()) {
                    item {
                        EmptyContentView(
                            type = nftEmptyContentType(
                                showReceiveAction = showReceiveAction,
                                onAction = onAction,
                            ),
                            modifier = Modifier.fillParentMaxSize(),
                        )
                    }
                }
                return@PullToRefreshBox
            }

            Column(modifier = Modifier.fillMaxSize()) {
                LazyVerticalGrid(
                    modifier = Modifier
                        .then(if (items.isEmpty()) Modifier else Modifier.weight(1f))
                        .fillMaxWidth(),
                    columns = GridCells.Adaptive(minSize = 150.dp),
                    state = listState,
                    contentPadding = PaddingValues(paddingSmall, paddingDefault)
                ) {
                    items(items) { item ->
                        NFTItem(
                            model = item,
                            onClick = {
                                val asset = item.asset
                                if (asset == null) {
                                    onAction(NftListAction.OpenCollection(item.collection.id.toIdentifier()))
                                } else {
                                    onAction(NftListAction.OpenAsset(asset.id))
                                }
                            },
                        )
                    }
                }
                if (showUnverifiedRow) {
                    LinkItem(
                        title = stringResource(R.string.asset_verification_unverified),
                        listPosition = ListPosition.Single,
                        trailingContent = {
                            PropertyDataText(
                                text = unverifiedCount.toString(),
                                badge = { DataBadgeChevron() },
                            )
                        },
                        onClick = { onAction(NftListAction.OpenUnverified) },
                    )
                }
            }
        }
    }
}

private fun nftEmptyContentType(
    showReceiveAction: Boolean,
    onAction: (NftListAction) -> Unit,
): EmptyContentType.Nft {
    val onReceive: (() -> Unit)? = if (showReceiveAction) {
        { onAction(NftListAction.Receive) }
    } else {
        null
    }

    return EmptyContentType.Nft(onReceive = onReceive)
}
