package com.gemwallet.android.features.assets.views

import androidx.compose.foundation.layout.fillMaxSize
import uniffi.gemstone.GemAssetAction
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.asset_select.presents.views.assetRows
import com.gemwallet.android.features.asset_select.presents.views.getAssetBadge
import com.gemwallet.android.features.asset_select.presents.views.searchState
import com.gemwallet.android.features.assets.viewmodels.AssetsResultsViewModel
import com.gemwallet.android.features.perpetual.views.components.PerpetualItem
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.AssetContextActions
import com.gemwallet.android.ui.components.list_item.PinnedAssetsHeaderItem
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.assetPriceSupport
import com.gemwallet.android.ui.components.list_item.getBalanceInfo
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.AssetToastEffect
import com.gemwallet.android.ui.components.screen.PullToRefreshBox
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.AssetsGroupType
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PerpetualId

@Composable
fun AssetsResultsScreen(
    onAction: (WalletSearchAction) -> Unit,
    viewModel: AssetsResultsViewModel = hiltViewModel(),
) {
    val pinned by viewModel.pinned.collectAsStateWithLifecycle()
    val cappedAssets by viewModel.cappedAssets.collectAsStateWithLifecycle()
    val previewPerpetuals by viewModel.previewPerpetuals.collectAsStateWithLifecycle()
    val state by viewModel.state.collectAsStateWithLifecycle()
    val isRefreshing by viewModel.refreshing.collectAsStateWithLifecycle()
    val longPressedAsset = remember { mutableStateOf<AssetId?>(null) }
    val longPressedPerpetual = remember { mutableStateOf<PerpetualId?>(null) }
    val snackbar = remember { SnackbarHostState() }
    AssetToastEffect(viewModel.toastEvents, snackbar)

    val onAssetClick: (Asset) -> Unit = {
        viewModel.updateRecent(it, GemAssetAction.OPEN)
        onAction(WalletSearchAction.OpenAsset(it))
    }
    val onPerpetualClick: (Asset) -> Unit = {
        viewModel.updateRecent(it, GemAssetAction.OPEN)
        onAction(WalletSearchAction.OpenPerpetual(it))
    }
    val contextActions = remember(viewModel) {
        AssetContextActions(
            onTogglePin = viewModel::onTogglePin,
            onAddToWallet = viewModel::onAddToWallet,
        )
    }
    Scene(
        title = viewModel.title,
        snackbar = snackbar,
        onClose = { onAction(WalletSearchAction.Cancel) },
    ) {
        PullToRefreshBox(
            modifier = Modifier.fillMaxSize(),
            isRefreshing = isRefreshing,
            onRefresh = viewModel::refresh,
        ) {
            LazyColumn(modifier = Modifier.fillMaxSize()) {
                if (pinned.isNotEmpty()) {
                    item { PinnedAssetsHeaderItem(AssetsGroupType.Pinned) }
                    assetRows(
                        items = pinned,
                        onSelect = onAssetClick,
                        support = { assetPriceSupport(it.price) },
                        titleBadge = ::getAssetBadge,
                        itemTrailing = { getBalanceInfo(it)() },
                        longPressedAsset = longPressedAsset,
                        contextActions = contextActions,
                    )
                }
                assetRows(
                    items = cappedAssets,
                    onSelect = onAssetClick,
                    support = { assetPriceSupport(it.price) },
                    titleBadge = ::getAssetBadge,
                    itemTrailing = { getBalanceInfo(it)() },
                    longPressedAsset = longPressedAsset,
                    contextActions = contextActions,
                )
                if (previewPerpetuals.isNotEmpty()) {
                    item { SubheaderItem(R.string.perpetuals_title) }
                    itemsPositioned(previewPerpetuals) { position, item ->
                        PerpetualItem(
                            item = item,
                            listPosition = position,
                            longPressState = longPressedPerpetual,
                            onTogglePin = viewModel::onTogglePerpetualPin,
                            onClick = { onPerpetualClick(item.asset) },
                        )
                    }
                }
                searchState(state)
            }
        }
    }
}
