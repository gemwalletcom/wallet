package com.gemwallet.android.features.swap.views

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.type
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.AssetItemUIModel
import com.gemwallet.android.ui.components.list_item.ListItemSupportText
import com.gemwallet.android.ui.components.list_item.getBalanceInfo
import com.gemwallet.android.features.asset_select.presents.views.AssetSelectAction
import com.gemwallet.android.features.asset_select.presents.views.AssetSelectScene
import com.gemwallet.android.features.asset_select.presents.views.RecentsSheetHost
import com.gemwallet.android.features.asset_select.viewmodels.RecentsSheetViewModel
import com.gemwallet.android.features.swap.viewmodels.SwapSelectViewModel
import com.gemwallet.android.domains.swap.SwapItemType
import com.gemwallet.android.model.RecentType
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetSubtype
import kotlinx.collections.immutable.toImmutableList

@Composable
fun SwapSelectScreen(
    onCancel: () -> Unit,
    onSelect: (select: SwapItemType, payId: AssetId?, receiveId: AssetId?) -> Unit,
    viewModel: SwapSelectViewModel = hiltViewModel(),
    recentsViewModel: RecentsSheetViewModel = hiltViewModel(),
) {
    val uiStates by viewModel.uiState.collectAsStateWithLifecycle()
    val pinned by viewModel.pinned.collectAsStateWithLifecycle()
    val unpinned by viewModel.unpinned.collectAsStateWithLifecycle()
    val recent by viewModel.recent.collectAsStateWithLifecycle()
    val availableChains by viewModel.availableChains.collectAsStateWithLifecycle()
    val chainsFilter by viewModel.chainFilter.collectAsStateWithLifecycle()
    val balanceFilter by viewModel.balanceFilter.collectAsStateWithLifecycle()
    val select by viewModel.select.collectAsStateWithLifecycle()
    val payId by viewModel.payAssetId.collectAsStateWithLifecycle()
    val receiveId by viewModel.receiveAssetId.collectAsStateWithLifecycle()
    val selectedTag by viewModel.selectedTag.collectAsStateWithLifecycle()

    val onSelectAsset: (AssetId) -> Unit = { assetId ->
        when (select) {
            SwapItemType.Pay -> onSelect(SwapItemType.Pay, assetId, receiveId)
            SwapItemType.Receive -> onSelect(SwapItemType.Receive, payId, assetId)
        }
    }

    AssetSelectScene(
        title = when (select) {
            SwapItemType.Pay -> stringResource(id = R.string.swap_you_pay)
            SwapItemType.Receive -> stringResource(id = R.string.swap_you_receive)
        },
        titleBadge = { null },
        query = viewModel.queryState,
        tags = viewModel.getTags(),
        selectedTag = selectedTag,
        popular = emptyList<AssetItemUIModel>().toImmutableList(),
        pinned = pinned,
        unpinned = unpinned,
        recent = recent,
        state = uiStates,
        availableChains = availableChains,
        chainsFilter = chainsFilter,
        balanceFilter = balanceFilter,
        onAction = { action ->
            when (action) {
                is AssetSelectAction.ChainFilter -> viewModel.onChainFilter(action.chain)
                is AssetSelectAction.BalanceFilter -> viewModel.onBalanceFilter(action.onlyWithBalance)
                AssetSelectAction.ClearFilters -> viewModel.onClearFilters()
                is AssetSelectAction.Select -> {
                    viewModel.updateRecent(action.assetId, RecentType.SwapSelect)
                    onSelectAsset(action.assetId)
                }
                is AssetSelectAction.SelectRecent -> onSelectAsset(action.assetId)
                AssetSelectAction.OpenRecentsSheet -> recentsViewModel.show(filters = viewModel.assetFilters(), types = viewModel.recentTypes)
                AssetSelectAction.Cancel -> onCancel()
                is AssetSelectAction.SelectTag -> viewModel.onTagSelect(action.tag)
                AssetSelectAction.AddAsset,
                AssetSelectAction.ShowAllAssets -> Unit
            }
        },
        recentsSheetEnabled = true,
        itemTrailing = { getBalanceInfo(it)() },
        support = {
            if (it.asset.id.type() == AssetSubtype.NATIVE) null else {
                @Composable { ListItemSupportText(it.asset.id.chain.asset().name) }
            }
        },
    )

    RecentsSheetHost(viewModel = recentsViewModel, onSelect = onSelectAsset)
}
