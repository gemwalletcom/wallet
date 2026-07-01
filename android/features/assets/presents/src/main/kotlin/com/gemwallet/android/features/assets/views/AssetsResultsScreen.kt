package com.gemwallet.android.features.assets.views

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.material3.pulltorefresh.PullToRefreshDefaults.Indicator
import androidx.compose.material3.pulltorefresh.rememberPullToRefreshState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.asset_select.presents.views.assetRows
import com.gemwallet.android.features.asset_select.presents.views.getAssetBadge
import com.gemwallet.android.features.asset_select.presents.views.searchState
import com.gemwallet.android.features.assets.viewmodels.AssetsResultsViewModel
import com.gemwallet.android.features.perpetual.views.components.PerpetualItem
import com.gemwallet.android.model.RecentType
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.AssetContextActions
import com.gemwallet.android.ui.components.list_item.PinnedAssetsHeaderItem
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.assetPriceSupport
import com.gemwallet.android.ui.components.list_item.getBalanceInfo
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.AssetsGroupType
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

    val onAssetClick: (AssetId) -> Unit = {
        viewModel.updateRecent(it, RecentType.Search)
        onAction(WalletSearchAction.OpenAsset(it))
    }
    val onPerpetualClick: (AssetId) -> Unit = {
        viewModel.onOpenPerpetual(it)
        onAction(WalletSearchAction.OpenPerpetual(it))
    }
    val contextActions = remember(viewModel) {
        AssetContextActions(
            onTogglePin = viewModel::onTogglePin,
            onAddToWallet = { id -> viewModel.onChangeVisibility(id, true) },
        )
    }
    val pullToRefreshState = rememberPullToRefreshState()

    Scene(
        title = viewModel.title,
        onClose = { onAction(WalletSearchAction.Cancel) },
    ) {
        PullToRefreshBox(
            modifier = Modifier.fillMaxSize(),
            isRefreshing = isRefreshing,
            onRefresh = viewModel::refresh,
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
                            onClick = onPerpetualClick,
                        )
                    }
                }
                searchState(state)
            }
        }
    }
}
