package com.gemwallet.android.features.asset_select.presents.views

import androidx.compose.foundation.layout.RowScope
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.ext.networkName
import com.gemwallet.android.ext.type
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.ui.components.list_item.ListItemSupportText
import com.gemwallet.android.ui.components.screen.SceneTitle
import com.gemwallet.android.features.asset_select.viewmodels.BaseAssetSelectViewModel
import com.gemwallet.android.features.asset_select.viewmodels.RecentsSheetViewModel
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetSubtype
import kotlinx.collections.immutable.toImmutableList

@Composable
fun AssetSelectScreen(
    title: String = "",
    titleContent: (@Composable () -> Unit)? = null,
    closeIcon: Boolean = false,
    titleBadge: (AssetInfoDataAggregate) -> String?,
    onCancel: () -> Unit,
    onSelect: ((AssetId) -> Unit)? = null,
    onSelectRecent: ((AssetId) -> Unit)? = null,
    itemTrailing: (@Composable (AssetInfoDataAggregate) -> Unit)? = null,
    itemSupport: ((AssetInfoDataAggregate) -> (@Composable () -> Unit)?)? = null,
    onAddAsset: (() -> Unit)? = null,
    showFilter: Boolean? = null,
    actions: @Composable RowScope.() -> Unit = {},
    viewModel: BaseAssetSelectViewModel,
    recentsViewModel: RecentsSheetViewModel = hiltViewModel(),
) {
    val flow = viewModel.flow
    val showRecents = flow.recents && onSelectRecent != null
    val uiStates by viewModel.uiState.collectAsStateWithLifecycle()
    val popular by viewModel.popular.collectAsStateWithLifecycle()
    val pinned by viewModel.pinned.collectAsStateWithLifecycle()
    val unpinned by viewModel.unpinned.collectAsStateWithLifecycle()
    val recent by viewModel.recent.collectAsStateWithLifecycle()
    val isAddAvailable by viewModel.isAddAssetAvailable.collectAsStateWithLifecycle()
    val availableChains by viewModel.availableChains.collectAsStateWithLifecycle()
    val chainsFilter by viewModel.chainFilter.collectAsStateWithLifecycle()
    val balanceFilter by viewModel.balanceFilter.collectAsStateWithLifecycle()

    val selectAsset: ((Asset) -> Unit)? = onSelect?.let { select ->
        { asset ->
            viewModel.onSelected(asset)
            select(asset.id)
        }
    }

    AssetSelectScene(
        title = titleContent ?: { SceneTitle(title) },
        titleBadge = titleBadge,
        closeIcon = closeIcon,
        support = if (itemSupport == null) {
            {
                if (it.asset.id.type() == AssetSubtype.NATIVE) null else {
                    @Composable { ListItemSupportText(it.asset.id.chain.networkName()) }
                }
            }
        } else {
            itemSupport
        },
        query = viewModel.queryState,
        pinned = pinned,
        popular = popular,
        unpinned = unpinned,
        recent = if (showRecents) recent else emptyList<Asset>().toImmutableList(),
        state = uiStates,
        isAddAvailable = isAddAvailable && onAddAsset != null,
        availableChains = availableChains,
        chainsFilter = chainsFilter,
        balanceFilter = balanceFilter,
        showFilter = showFilter ?: flow.chainFilter,
        showBalanceFilter = flow.balanceFilter,
        onAction = { action ->
            when (action) {
                is AssetSelectAction.ChainFilter -> viewModel.onChainFilter(action.chain)
                is AssetSelectAction.BalanceFilter -> viewModel.onBalanceFilter(action.onlyWithBalance)
                AssetSelectAction.ClearFilters -> viewModel.onClearFilters()
                is AssetSelectAction.Select -> selectAsset?.invoke(action.asset)
                is AssetSelectAction.SelectRecent -> onSelectRecent?.invoke(action.asset.id)
                AssetSelectAction.OpenRecentsSheet -> recentsViewModel.show(filters = viewModel.assetFilters(), types = viewModel.recentTypes)
                AssetSelectAction.Cancel -> onCancel()
                AssetSelectAction.AddAsset -> onAddAsset?.invoke()
                AssetSelectAction.ShowAllAssets -> Unit
            }
        },
        recentsSheetEnabled = showRecents,
        itemTrailing = itemTrailing,
        actions = actions,
    )

    if (showRecents) {
        RecentsSheetHost(viewModel = recentsViewModel, onSelect = { onSelectRecent(it.id) })
    }
}
