package com.gemwallet.android.features.nft.presents

import androidx.compose.foundation.layout.Box
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
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
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
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.cases.nft.NftError
import com.gemwallet.android.features.nft.presents.components.NFTItem
import com.gemwallet.android.features.nft.viewmodels.NftListViewModels
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.empty.EmptyContentType
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.list_item.LinkItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.NftItemUIModel
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.models.actions.NftAssetIdAction
import com.gemwallet.android.ui.models.actions.NftCollectionIdAction
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall

@Composable
fun NftListNavScreen(
    cancelAction: CancelAction?,
    collectionAction: NftCollectionIdAction,
    assetAction: NftAssetIdAction,
    onReceive: (() -> Unit)? = null,
    listState: LazyGridState = rememberLazyGridState(),
    title: String = stringResource(R.string.nft_collections),
    onUnverifiedClick: (() -> Unit)? = null,
    viewModel: NftListViewModels = hiltViewModel(),
) {
    val items by viewModel.collections.collectAsStateWithLifecycle()
    val isRefreshing by viewModel.isRefreshing.collectAsStateWithLifecycle()
    val error by viewModel.error.collectAsStateWithLifecycle()
    val unverifiedCount by viewModel.unverifiedCount.collectAsStateWithLifecycle()
    val walletId by viewModel.walletId.collectAsStateWithLifecycle()

    LaunchedEffect(walletId) {
        viewModel.syncIfNeeded()
    }

    NftListScene(
        items = items,
        isRefreshing = isRefreshing,
        error = error,
        unverifiedCount = unverifiedCount,
        title = title,
        showCloseAction = cancelAction != null,
        showReceiveAction = onReceive != null,
        showUnverifiedAction = onUnverifiedClick != null,
        listState = listState,
        onAction = { action ->
            when (action) {
                NftListAction.Refresh -> viewModel.refresh()
                NftListAction.Close -> cancelAction?.invoke()
                NftListAction.Receive -> onReceive?.invoke()
                NftListAction.OpenUnverified -> onUnverifiedClick?.invoke()
                is NftListAction.OpenCollection -> collectionAction(action.collectionId)
                is NftListAction.OpenAsset -> assetAction(action.assetId)
            }
        },
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun NftListScene(
    items: List<NftItemUIModel>,
    isRefreshing: Boolean,
    error: NftError?,
    unverifiedCount: Int,
    title: String,
    showCloseAction: Boolean,
    showReceiveAction: Boolean,
    showUnverifiedAction: Boolean,
    listState: LazyGridState = rememberLazyGridState(),
    onAction: (NftListAction) -> Unit,
) {
    val pullToRefreshState = rememberPullToRefreshState()

    Scene(
        title = title,
        navigationBarPadding = false,
        actions = {
            if (showReceiveAction) {
                IconButton(onClick = { onAction(NftListAction.Receive) }) {
                    Icon(
                        imageVector = Icons.Default.Add,
                        contentDescription = stringResource(R.string.wallet_receive),
                    )
                }
            }
        },
        onClose = if (showCloseAction) {
            { onAction(NftListAction.Close) }
        } else {
            null
        }
    ) {
        PullToRefreshBox(
            modifier = Modifier.fillMaxSize(),
            isRefreshing = isRefreshing,
            onRefresh = { onAction(NftListAction.Refresh) },
            state = pullToRefreshState,
            indicator = {
                Indicator(
                    modifier = Modifier.align(Alignment.TopCenter),
                    isRefreshing = isRefreshing,
                    state = pullToRefreshState,
                    containerColor = MaterialTheme.colorScheme.background,
                )
            },
        ) {
            val currentError = error
            if (currentError != null) {
                Box(modifier = Modifier.fillMaxSize()) {
                    Column(modifier = Modifier.align(Alignment.Center), horizontalAlignment = Alignment.CenterHorizontally) {
                        Text(
                            textAlign = TextAlign.Center,
                            text = when (currentError) {
                                NftError.LoadError -> stringResource(R.string.errors_error_occured)
                                NftError.NotFoundAsset -> currentError.message.orEmpty()
                                NftError.NotFoundCollection -> currentError.message.orEmpty()
                            }
                        )
                        TextButton(onClick = { onAction(NftListAction.Refresh) }) {
                            Text(stringResource(R.string.common_try_again))
                        }
                    }
                }
                return@PullToRefreshBox
            }

            val showUnverifiedRow = showUnverifiedAction && unverifiedCount > 0

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
                    modifier = Modifier.weight(1f).fillMaxWidth(),
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
                                    onAction(NftListAction.OpenCollection(item.collection.id))
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
