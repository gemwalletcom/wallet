package com.gemwallet.android.features.wallet_tab.presents

import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.wallet_tab.viewmodels.WalletViewModel
import com.gemwallet.android.ui.components.list_item.AssetContextActions
import com.gemwallet.android.ui.components.screen.ToastEffect

@Composable
fun WalletScreen(onAction: (WalletAction) -> Unit, onContentReady: () -> Unit = {}, listState: LazyListState = rememberLazyListState(), viewModel: WalletViewModel = hiltViewModel()) {
    val importing by viewModel.isLoadingAssets.collectAsStateWithLifecycle()
    val pinnedAssets by viewModel.pinnedAssets.collectAsStateWithLifecycle()
    val unpinnedAssets by viewModel.unpinnedAssets.collectAsStateWithLifecycle()
    val walletSummary by viewModel.walletSummary.collectAsStateWithLifecycle()
    val bannerRow by viewModel.bannerRow.collectAsStateWithLifecycle()
    val isRefreshing by viewModel.isRefreshing.collectAsStateWithLifecycle()
    val collectionsAvailable by viewModel.collectionsAvailable.collectAsStateWithLifecycle()

    val snackbar = remember { SnackbarHostState() }
    ToastEffect(viewModel.toastEvents, snackbar)

    val currentOnContentReady by rememberUpdatedState(onContentReady)
    LaunchedEffect(walletSummary != null) {
        if (walletSummary != null) currentOnContentReady()
    }

    val currentWalletId by viewModel.currentWalletId.collectAsStateWithLifecycle()
    var previousWalletId by rememberSaveable { mutableStateOf<String?>(null) }
    LaunchedEffect(currentWalletId) {
        val walletId = currentWalletId?.id ?: return@LaunchedEffect
        if (previousWalletId != null && previousWalletId != walletId) {
            listState.scrollToItem(0)
        }
        previousWalletId = walletId
    }

    val assetActions = remember(viewModel) {
        AssetContextActions(
            onTogglePin = viewModel::togglePin,
            onHide = viewModel::hideAsset,
        )
    }

    WalletScene(
        walletSummary = walletSummary,
        importing = importing,
        pinnedAssets = pinnedAssets,
        unpinnedAssets = unpinnedAssets,
        bannerRow = bannerRow,
        isRefreshing = isRefreshing,
        collectionsAvailable = collectionsAvailable,
        snackbar = snackbar,
        listState = listState,
        assetActions = assetActions,
        onRefresh = viewModel::onRefresh,
        onHideBalances = viewModel::hideBalances,
        onCloseBanner = viewModel::closeBanner,
        onAction = onAction,
    )
}
