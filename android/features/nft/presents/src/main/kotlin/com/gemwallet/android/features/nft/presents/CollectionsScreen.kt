package com.gemwallet.android.features.nft.presents

import androidx.compose.foundation.lazy.grid.LazyGridState
import androidx.compose.foundation.lazy.grid.rememberLazyGridState
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.nft.viewmodels.CollectionsViewModel
import com.gemwallet.android.ui.components.screen.ToastEffect
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.models.actions.NftAssetIdAction
import com.gemwallet.android.ui.models.actions.NftCollectionIdAction

@Composable
fun CollectionsScreen(
    cancelAction: CancelAction,
    collectionAction: NftCollectionIdAction,
    assetAction: NftAssetIdAction,
    onReceive: () -> Unit,
    onUnverified: () -> Unit,
    listState: LazyGridState = rememberLazyGridState(),
    viewModel: CollectionsViewModel = hiltViewModel(),
) {
    val items by viewModel.collections.collectAsStateWithLifecycle()
    val isRefreshing by viewModel.isRefreshing.collectAsStateWithLifecycle()
    val unverifiedListItem by viewModel.unverifiedListItem.collectAsStateWithLifecycle()
    val errorRow by viewModel.errorRow.collectAsStateWithLifecycle()
    val walletId by viewModel.walletId.collectAsStateWithLifecycle()
    val title by viewModel.title.collectAsStateWithLifecycle()
    val showReceiveAction by viewModel.showReceiveAction.collectAsStateWithLifecycle()
    val snackbar = remember { SnackbarHostState() }
    ToastEffect(viewModel.toastEvents, snackbar)

    LaunchedEffect(walletId) {
        viewModel.syncIfNeeded()
    }

    CollectionsScene(
        items = items,
        isRefreshing = isRefreshing,
        errorRow = errorRow,
        unverifiedListItem = unverifiedListItem,
        title = title,
        showReceiveAction = showReceiveAction,
        listState = listState,
        snackbar = snackbar,
        onAction = { action ->
            when (action) {
                CollectionsAction.Refresh -> viewModel.refresh()
                CollectionsAction.Close -> cancelAction()
                CollectionsAction.Receive -> onReceive()
                CollectionsAction.OpenUnverified -> onUnverified()
                is CollectionsAction.OpenCollection -> collectionAction(action.collectionId)
                is CollectionsAction.OpenAsset -> assetAction(action.assetId)
            }
        },
    )
}
