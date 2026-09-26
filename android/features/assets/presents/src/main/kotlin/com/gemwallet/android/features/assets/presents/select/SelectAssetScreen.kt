package com.gemwallet.android.features.assets.presents.select

import androidx.compose.foundation.layout.RowScope
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.platform.LocalContext
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.assets.viewmodels.select.BaseSelectAssetViewModel
import com.gemwallet.android.features.assets.viewmodels.select.RecentsViewModel
import com.gemwallet.android.ui.components.clipboard.clipboardManager
import com.gemwallet.android.ui.components.clipboard.setCopy
import com.gemwallet.android.ui.components.list_item.AssetItemAction
import com.gemwallet.android.ui.components.screen.SceneTitle
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import kotlinx.collections.immutable.toImmutableList

@Composable
fun SelectAssetScreen(
    titleContent: (@Composable () -> Unit)? = null,
    closeIcon: Boolean = false,
    onCancel: () -> Unit,
    onSelect: ((AssetId) -> Unit)? = null,
    onSelectRecent: ((AssetId) -> Unit)? = null,
    onAddAsset: (() -> Unit)? = null,
    showFilter: Boolean? = null,
    actions: @Composable RowScope.() -> Unit = {},
    viewModel: BaseSelectAssetViewModel,
    recentsViewModel: RecentsViewModel = hiltViewModel(),
) {
    val flow = viewModel.flowUIModel
    val title = flow.title
    val context = LocalContext.current
    val clipboardManager = LocalContext.current.clipboardManager()
    val uiStates by viewModel.uiState.collectAsStateWithLifecycle()
    val popular by viewModel.popular.collectAsStateWithLifecycle()
    val pinned by viewModel.pinned.collectAsStateWithLifecycle()
    val unpinned by viewModel.unpinned.collectAsStateWithLifecycle()
    val recent by viewModel.recent.collectAsStateWithLifecycle()
    val showsRecents by viewModel.showsRecents.collectAsStateWithLifecycle()
    val showRecents = showsRecents && onSelectRecent != null
    val isAddAvailable by viewModel.isAddAssetAvailable.collectAsStateWithLifecycle()
    val isChainFilterAvailable by viewModel.isChainFilterAvailable.collectAsStateWithLifecycle()
    val availableChains by viewModel.availableChains.collectAsStateWithLifecycle()
    val chainsFilter by viewModel.chainFilter.collectAsStateWithLifecycle()
    val balanceFilter by viewModel.balanceFilter.collectAsStateWithLifecycle()

    val selectAsset: ((Asset) -> Unit)? = onSelect?.let { select ->
        { asset ->
            viewModel.onSelected(asset)
            select(asset.id)
        }
    }

    SelectAssetScene(
        title = titleContent ?: { SceneTitle(title) },
        closeIcon = closeIcon,
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
        showFilter = showFilter ?: isChainFilterAvailable,
        showBalanceFilter = flow.showsBalanceFilter,
        onAction = { action ->
            when (action) {
                is SelectAssetAction.ChainFilter -> viewModel.onChainFilter(action.chain)
                is SelectAssetAction.BalanceFilter -> viewModel.onBalanceFilter(action.onlyWithBalance)
                SelectAssetAction.ClearFilters -> viewModel.onClearFilters()
                is SelectAssetAction.Select -> selectAsset?.invoke(action.asset)
                is SelectAssetAction.SelectRecent -> onSelectRecent?.invoke(action.asset.id)
                SelectAssetAction.OpenRecentsSheet -> recentsViewModel.show(filters = viewModel.assetFilters(), types = viewModel.recentTypes)
                SelectAssetAction.Cancel -> onCancel()
                SelectAssetAction.AddAsset -> onAddAsset?.invoke()
                SelectAssetAction.ShowAllAssets -> Unit
            }
        },
        recentsSheetEnabled = showRecents,
        onItemAction = { item, action ->
            when (action) {
                is AssetItemAction.Switch -> viewModel.onChangeVisibility(item.asset.id, action.enabled)
                AssetItemAction.Copy -> clipboardManager.setCopy(context, viewModel.addressCopy(item))
            }
        },
        actions = actions,
    )

    if (showRecents) {
        RecentsScreen(viewModel = recentsViewModel, onSelect = { onSelectRecent(it.id) })
    }
}
