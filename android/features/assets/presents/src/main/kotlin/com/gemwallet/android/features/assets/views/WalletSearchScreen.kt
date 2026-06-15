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
import com.gemwallet.android.features.asset_select.presents.views.AssetSelectAction
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

    val handleAction: (WalletSearchAction) -> Unit = { action ->
        when (action) {
            is WalletSearchAction.PinAsset -> viewModel.onPinAsset(action.assetId)
            is WalletSearchAction.AddToWallet -> viewModel.onChangeVisibility(action.assetId, true)
            is WalletSearchAction.TogglePerpetualPin -> viewModel.onTogglePerpetualPin(action.perpetualId)
            is WalletSearchAction.SelectTag -> viewModel.onTagSelect(action.tag)
            WalletSearchAction.OpenRecentsSheet -> recentsViewModel.show(filters = viewModel.assetFilters())
            is WalletSearchAction.OpenRecent -> onAction(
                if (action.assetId.toIdentifier() in perpetualRecentIds) {
                    WalletSearchAction.OpenPerpetual(action.assetId)
                } else {
                    WalletSearchAction.OpenAsset(action.assetId)
                }
            )
            is WalletSearchAction.OpenAsset -> {
                viewModel.updateRecent(action.assetId, RecentType.Search)
                onAction(action)
            }
            is WalletSearchAction.OpenPerpetual -> {
                viewModel.onOpenPerpetual(action.assetId)
                onAction(action)
            }
            WalletSearchAction.AddAsset,
            WalletSearchAction.Cancel,
            WalletSearchAction.OpenPerpetuals,
            is WalletSearchAction.ShowAllAssets -> onAction(action)
        }
    }

    val pinnedPerpetualRows: List<@Composable (ListPosition) -> Unit> = pinnedPerpetuals.map { item ->
        @Composable { position: ListPosition ->
            PerpetualItem(
                item = item,
                listPosition = position,
                longPressState = longPressedPerpetual,
                onTogglePin = { handleAction(WalletSearchAction.TogglePerpetualPin(it)) },
                onClick = { handleAction(WalletSearchAction.OpenPerpetual(it)) },
            )
        }
    }

    val perpetualsContent: (LazyListScope.() -> Unit)? = if (previewPerpetuals.isNotEmpty()) {
        {
            item {
                SubheaderItem(R.string.perpetuals_title, if (hasMorePerpetuals) ({ handleAction(WalletSearchAction.OpenPerpetuals) }) else null)
            }
            itemsPositioned(previewPerpetuals) { position, item ->
                PerpetualItem(
                    item = item,
                    listPosition = position,
                    longPressState = longPressedPerpetual,
                    onTogglePin = { handleAction(WalletSearchAction.TogglePerpetualPin(it)) },
                    onClick = { handleAction(WalletSearchAction.OpenPerpetual(it)) },
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
        onAction = { action ->
            when (action) {
                AssetSelectAction.Cancel -> handleAction(WalletSearchAction.Cancel)
                AssetSelectAction.AddAsset -> handleAction(WalletSearchAction.AddAsset)
                AssetSelectAction.OpenRecentsSheet -> handleAction(WalletSearchAction.OpenRecentsSheet)
                AssetSelectAction.ShowAllAssets -> handleAction(
                    WalletSearchAction.ShowAllAssets(viewModel.queryState.text.toString(), selectedTag)
                )
                is AssetSelectAction.Select -> handleAction(WalletSearchAction.OpenAsset(action.assetId))
                is AssetSelectAction.SelectRecent -> handleAction(WalletSearchAction.OpenRecent(action.assetId))
                is AssetSelectAction.SelectTag -> handleAction(WalletSearchAction.SelectTag(action.tag))
                is AssetSelectAction.ChainFilter,
                is AssetSelectAction.BalanceFilter,
                AssetSelectAction.ClearFilters -> Unit
            }
        },
        recentsSheetEnabled = true,
        itemTrailing = { asset -> getBalanceInfo(asset)() },
        contextActions = AssetContextActions(
            onTogglePin = { handleAction(WalletSearchAction.PinAsset(it)) },
            onAddToWallet = { handleAction(WalletSearchAction.AddToWallet(it)) },
        ),
        pinnedPerpetualRows = pinnedPerpetualRows,
        perpetualsContent = perpetualsContent,
        assetsHeaderRes = R.string.assets_title,
        assetsHeaderClickable = hasMoreAssets,
    )

    RecentsSheetHost(viewModel = recentsViewModel, onSelect = { handleAction(WalletSearchAction.OpenRecent(it)) })
}
