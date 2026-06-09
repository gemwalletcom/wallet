package com.gemwallet.android.features.assets.views

import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.asset_select.presents.views.AssetSelectScene
import com.gemwallet.android.features.asset_select.presents.views.RecentsSheetHost
import com.gemwallet.android.features.asset_select.presents.views.getAssetBadge
import com.gemwallet.android.features.asset_select.viewmodels.RecentsSheetViewModel
import com.gemwallet.android.features.assets.viewmodels.WalletSearchViewModel
import com.gemwallet.android.features.perpetual.views.components.PerpetualItem
import com.gemwallet.android.model.RecentType
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.SearchBar
import com.gemwallet.android.ui.components.list_item.AssetContextActions
import com.gemwallet.android.ui.components.list_item.AssetItemUIModel
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.assetPriceSupport
import com.gemwallet.android.ui.components.list_item.getBalanceInfo
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.models.ListPosition
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PerpetualId
import kotlinx.collections.immutable.toImmutableList

@Composable
fun WalletSearchScreen(
    onAction: (WalletSearchAction) -> Unit,
    viewModel: WalletSearchViewModel = hiltViewModel(),
    recentsViewModel: RecentsSheetViewModel = hiltViewModel(),
) {
    val isAddAssetAvailable by viewModel.isAddAssetAvailable.collectAsStateWithLifecycle()
    val state by viewModel.state.collectAsStateWithLifecycle()
    val pinned by viewModel.pinned.collectAsStateWithLifecycle()
    val previewAssets by viewModel.previewAssets.collectAsStateWithLifecycle()
    val hasMoreAssets by viewModel.hasMoreAssets.collectAsStateWithLifecycle()
    val recent by viewModel.recent.collectAsStateWithLifecycle()
    val selectedTag by viewModel.selectedTag.collectAsStateWithLifecycle()
    val previewPerpetuals by viewModel.previewPerpetuals.collectAsStateWithLifecycle()
    val hasMorePerpetuals by viewModel.hasMorePerpetuals.collectAsStateWithLifecycle()
    val pinnedPerpetuals by viewModel.pinnedPerpetuals.collectAsStateWithLifecycle()
    val perpetualRecentIds by viewModel.perpetualRecentIds.collectAsStateWithLifecycle()

    val longPressedPerpetual = remember { mutableStateOf<PerpetualId?>(null) }

    val onRecentClick: (AssetId) -> Unit = { assetId ->
        onAction(if (assetId.toIdentifier() in perpetualRecentIds) WalletSearchAction.OpenPerpetual(assetId) else WalletSearchAction.OpenAsset(assetId))
    }

    val contextActions = remember(viewModel) {
        AssetContextActions(
            onTogglePin = viewModel::onPinAsset,
            onAddToWallet = { id -> viewModel.onChangeVisibility(id, true) },
        )
    }

    val selectAsset: (AssetId) -> Unit = { id ->
        viewModel.updateRecent(id, RecentType.Search)
        onAction(WalletSearchAction.OpenAsset(id))
    }

    val onPerpetualSelect: (AssetId) -> Unit = { assetId ->
        viewModel.onOpenPerpetual(assetId)
        onAction(WalletSearchAction.OpenPerpetual(assetId))
    }

    val pinnedPerpetualRows: List<@Composable (ListPosition) -> Unit> = pinnedPerpetuals.map { item ->
        @Composable { position: ListPosition ->
            PerpetualItem(
                item = item,
                listPosition = position,
                longPressState = longPressedPerpetual,
                onTogglePin = viewModel::onTogglePerpetualPin,
                onClick = onPerpetualSelect,
            )
        }
    }

    val perpetualsContent: (LazyListScope.() -> Unit)? = if (previewPerpetuals.isNotEmpty()) {
        {
            item {
                SubheaderItem(R.string.perpetuals_title, if (hasMorePerpetuals) ({ onAction(WalletSearchAction.OpenPerpetuals) }) else null)
            }
            itemsPositioned(previewPerpetuals) { position, item ->
                PerpetualItem(
                    item = item,
                    listPosition = position,
                    longPressState = longPressedPerpetual,
                    onTogglePin = viewModel::onTogglePerpetualPin,
                    onClick = onPerpetualSelect,
                )
            }
        }
    } else {
        null
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
        unpinned = previewAssets.toImmutableList(),
        recent = recent,
        state = state,
        isAddAvailable = isAddAssetAvailable,
        searchable = false,
        onChainFilter = {},
        onBalanceFilter = {},
        onClearFilters = {},
        onCancel = { onAction(WalletSearchAction.Cancel) },
        onAddAsset = if (isAddAssetAvailable) ({ onAction(WalletSearchAction.AddAsset) }) else null,
        onSelect = selectAsset,
        onSelectRecent = onRecentClick,
        onOpenRecentsSheet = { recentsViewModel.show(filters = viewModel.assetFilters()) },
        onTagSelect = viewModel::onTagSelect,
        itemTrailing = { asset -> getBalanceInfo(asset)() },
        contextActions = contextActions,
        pinnedPerpetualRows = pinnedPerpetualRows,
        perpetualsContent = perpetualsContent,
        assetsHeaderRes = R.string.assets_title,
        onAssetsHeaderClick = if (hasMoreAssets) {
            { onAction(WalletSearchAction.ShowAllAssets(viewModel.queryState.text.toString(), selectedTag)) }
        } else {
            null
        },
    )

    RecentsSheetHost(viewModel = recentsViewModel, onSelect = onRecentClick)
}
