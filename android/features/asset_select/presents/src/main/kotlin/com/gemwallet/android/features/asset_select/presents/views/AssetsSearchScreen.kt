package com.gemwallet.android.features.asset_select.presents.views

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.ui.components.SearchBar
import com.gemwallet.android.ui.components.list_item.AssetContextActions
import com.gemwallet.android.ui.components.list_item.AssetItemUIModel
import com.gemwallet.android.ui.components.list_item.assetPriceSupport
import com.gemwallet.android.ui.components.list_item.getBalanceInfo
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.features.asset_select.viewmodels.AssetSelectViewModel
import com.gemwallet.android.model.RecentType
import com.wallet.core.primitives.AssetId
import kotlinx.collections.immutable.toImmutableList

@Composable
fun AssetsSearchScreen(
    onAddAsset: () -> Unit,
    onAssetClick: (AssetId) -> Unit,
    onCancel: () -> Unit,
    viewModel: AssetSelectViewModel = hiltViewModel(),
) {
    val isAddAssetAvailable by viewModel.isAddAssetAvailable.collectAsStateWithLifecycle()
    val uiStates by viewModel.uiState.collectAsStateWithLifecycle()
    val pinned by viewModel.pinned.collectAsStateWithLifecycle()
    val unpinned by viewModel.unpinned.collectAsStateWithLifecycle()
    val recent by viewModel.recent.collectAsStateWithLifecycle()

    val selectedTag by viewModel.selectedTag.collectAsStateWithLifecycle()

    val contextActions = remember(viewModel) {
        AssetContextActions(
            onTogglePin = viewModel::onTogglePin,
            onAddToWallet = { id -> viewModel.onChangeVisibility(id, true) },
        )
    }

    val selectAsset: (AssetId) -> Unit = { id ->
        viewModel.updateRecent(id, RecentType.Search)
        onAssetClick(id)
    }

    AssetSelectScene(
        title = {
            SearchBar(
                query = viewModel.queryState,
                modifier = Modifier.listItem(ListPosition.Single, paddingHorizontal = 0.dp),
            )
        },
        titleBadge = ::getAssetBadge,
        support = { assetPriceSupport(it.price) },
        query = viewModel.queryState,
        selectedTag = selectedTag,
        tags = viewModel.getTags(),
        pinned = pinned,
        popular = emptyList<AssetItemUIModel>().toImmutableList(),
        unpinned = unpinned,
        recent = recent,
        state = uiStates,
        isAddAvailable = isAddAssetAvailable,
        availableChains = emptyList(),
        chainsFilter = emptyList(),
        balanceFilter = false,
        searchable = false,
        onChainFilter = {},
        onBalanceFilter = {},
        onClearFilters = {},
        onCancel = onCancel,
        onAddAsset = if (isAddAssetAvailable) onAddAsset else null,
        onSelect = selectAsset,
        onSelectRecent = onAssetClick,
        onTagSelect = viewModel::onTagSelect,
        itemTrailing = { asset -> getBalanceInfo(asset)() },
        contextActions = contextActions,
    )
}
