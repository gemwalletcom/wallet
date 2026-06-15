package com.gemwallet.android.features.asset_select.presents.views

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.type
import com.gemwallet.android.model.RecentType
import com.gemwallet.android.ui.components.list_item.AssetItemUIModel
import com.gemwallet.android.ui.components.list_item.ListItemSupportText
import com.gemwallet.android.features.asset_select.viewmodels.BaseAssetSelectViewModel
import com.gemwallet.android.features.asset_select.viewmodels.RecentsSheetViewModel
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetSubtype
import kotlinx.collections.immutable.toImmutableList

@Composable
fun AssetSelectScreen(
    title: String = "",
    titleBadge: (AssetItemUIModel) -> String?,
    showPopular: Boolean = false,
    recentType: RecentType? = null,
    onCancel: () -> Unit,
    onSelect: ((AssetId) -> Unit)? = null,
    onSelectRecent: ((AssetId) -> Unit)? = null,
    itemTrailing: (@Composable (AssetItemUIModel) -> Unit)? = null,
    itemSupport: ((AssetItemUIModel) -> (@Composable () -> Unit)?)? = null,
    onAddAsset: (() -> Unit)? = null,
    viewModel: BaseAssetSelectViewModel,
    recentsViewModel: RecentsSheetViewModel = hiltViewModel(),
) {
    val uiStates by viewModel.uiState.collectAsStateWithLifecycle()
    val popular by viewModel.popular.collectAsStateWithLifecycle()
    val pinned by viewModel.pinned.collectAsStateWithLifecycle()
    val unpinned by viewModel.unpinned.collectAsStateWithLifecycle()
    val recent by viewModel.recent.collectAsStateWithLifecycle()
    val isAddAvailable by viewModel.isAddAssetAvailable.collectAsStateWithLifecycle()
    val availableChains by viewModel.availableChains.collectAsStateWithLifecycle()
    val chainsFilter by viewModel.chainFilter.collectAsStateWithLifecycle()
    val balanceFilter by viewModel.balanceFilter.collectAsStateWithLifecycle()
    val selectedTag by viewModel.selectedTag.collectAsStateWithLifecycle()

    val selectAsset: ((AssetId) -> Unit)? = when {
        onSelect == null -> null
        recentType == null -> onSelect
        else -> { id -> viewModel.updateRecent(id, recentType); onSelect(id) }
    }

    AssetSelectScene(
        title = title,
        titleBadge = titleBadge,
        support = if (itemSupport == null) {
            {
                if (it.asset.id.type() == AssetSubtype.NATIVE) null else {
                    @Composable { ListItemSupportText(it.asset.id.chain.asset().name) }
                }
            }
        } else {
            itemSupport
        },
        query = viewModel.queryState,
        pinned = pinned,
        popular = if (showPopular) {
            popular
        } else {
            emptyList<AssetItemUIModel>().toImmutableList()
        },
        unpinned = unpinned,
        recent = recent,
        state = uiStates,
        isAddAvailable = isAddAvailable && onAddAsset != null,
        availableChains = availableChains,
        chainsFilter = chainsFilter,
        balanceFilter = balanceFilter,
        onAction = { action ->
            when (action) {
                is AssetSelectAction.ChainFilter -> viewModel.onChainFilter(action.chain)
                is AssetSelectAction.BalanceFilter -> viewModel.onBalanceFilter(action.onlyWithBalance)
                AssetSelectAction.ClearFilters -> viewModel.onClearFilters()
                is AssetSelectAction.Select -> selectAsset?.invoke(action.assetId)
                is AssetSelectAction.SelectRecent -> onSelectRecent?.invoke(action.assetId)
                AssetSelectAction.OpenRecentsSheet -> recentsViewModel.show(filters = viewModel.assetFilters())
                AssetSelectAction.Cancel -> onCancel()
                AssetSelectAction.AddAsset -> onAddAsset?.invoke()
                is AssetSelectAction.SelectTag -> viewModel.onTagSelect(action.tag)
                AssetSelectAction.ShowAllAssets -> Unit
            }
        },
        recentsSheetEnabled = onSelectRecent != null,
        itemTrailing = itemTrailing,
        selectedTag = selectedTag,
        tags = viewModel.getTags(),
    )

    if (onSelectRecent != null) {
        RecentsSheetHost(viewModel = recentsViewModel, onSelect = onSelectRecent)
    }
}
