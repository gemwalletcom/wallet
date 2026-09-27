package com.gemwallet.android.features.wallet_tab.presents

import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.assets.presents.select.RecentsScreen
import com.gemwallet.android.features.assets.presents.select.SelectAssetAction
import com.gemwallet.android.features.assets.presents.select.SelectAssetScene
import com.gemwallet.android.features.assets.viewmodels.select.RecentsViewModel
import com.gemwallet.android.features.perpetuals.presents.components.PerpetualListItem
import com.gemwallet.android.features.wallet_tab.viewmodels.WalletSearchViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.SearchBar
import com.gemwallet.android.ui.components.list_item.AssetContextActions
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemDefaults
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.NftListItem
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.ToastEffect
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.NftItemTarget
import com.gemwallet.android.ui.models.target
import com.gemwallet.android.ui.theme.space0
import com.wallet.core.primitives.PerpetualId
import kotlinx.collections.immutable.toImmutableList
import uniffi.gemstone.GemSearchListRow

@Composable
fun WalletSearchScreen(onAction: (WalletSearchAction) -> Unit, viewModel: WalletSearchViewModel = hiltViewModel(), recentsViewModel: RecentsViewModel = hiltViewModel()) {
    val searchEmptyState by viewModel.searchEmptyState.collectAsStateWithLifecycle()
    val state by viewModel.state.collectAsStateWithLifecycle()
    val pinned by viewModel.pinned.collectAsStateWithLifecycle()
    val previewAssets by viewModel.previewAssets.collectAsStateWithLifecycle()
    val hasMoreAssets by viewModel.hasMoreAssets.collectAsStateWithLifecycle()
    val recent by viewModel.recent.collectAsStateWithLifecycle()
    val previewPerpetuals by viewModel.previewPerpetuals.collectAsStateWithLifecycle()
    val hasMorePerpetuals by viewModel.hasMorePerpetuals.collectAsStateWithLifecycle()
    val pinnedPerpetuals by viewModel.pinnedPerpetuals.collectAsStateWithLifecycle()
    val previewNfts by viewModel.previewNfts.collectAsStateWithLifecycle()
    val hasMoreNfts by viewModel.hasMoreNfts.collectAsStateWithLifecycle()
    val lists by viewModel.lists.collectAsStateWithLifecycle()

    val longPressedPerpetual = remember { mutableStateOf<PerpetualId?>(null) }
    val snackbar = remember { SnackbarHostState() }
    ToastEffect(viewModel.toastEvents, snackbar)

    val handleAction: (WalletSearchAction) -> Unit = { action ->
        when (action) {
            is WalletSearchAction.PinAsset -> viewModel.onTogglePin(action.assetId)

            is WalletSearchAction.AddToWallet -> viewModel.onAddToWallet(action.assetId)

            is WalletSearchAction.TogglePerpetualPin -> viewModel.onTogglePerpetualPin(action.perpetualId)

            WalletSearchAction.OpenRecentsSheet -> recentsViewModel.show(filters = viewModel.assetFilters())

            is WalletSearchAction.OpenAsset -> {
                viewModel.openRecent(action.asset)
                onAction(action)
            }

            is WalletSearchAction.OpenPerpetual -> {
                viewModel.openRecent(action.asset)
                onAction(action)
            }

            WalletSearchAction.AddAsset,
            WalletSearchAction.Cancel,
            WalletSearchAction.OpenPerpetuals,
            WalletSearchAction.OpenCollections,
            is WalletSearchAction.OpenNftCollection,
            is WalletSearchAction.OpenNftAsset,
            is WalletSearchAction.OpenList,
            is WalletSearchAction.OpenRecent,
            is WalletSearchAction.ShowAllAssets,
            -> onAction(action)
        }
    }

    val pinnedPerpetualRows: List<@Composable (ListPosition) -> Unit> = pinnedPerpetuals.map { item ->
        @Composable { position: ListPosition ->
            PerpetualListItem(
                item = item,
                listPosition = position,
                longPressState = longPressedPerpetual,
                onTogglePin = { handleAction(WalletSearchAction.TogglePerpetualPin(it)) },
                onClick = { handleAction(WalletSearchAction.OpenPerpetual(item.data.asset.toPrimitives())) },
            )
        }
    }

    val perpetualsContent: (LazyListScope.() -> Unit)? = if (previewPerpetuals.isNotEmpty()) {
        {
            item {
                SubheaderItem(R.string.perpetuals_title, if (hasMorePerpetuals) ({ handleAction(WalletSearchAction.OpenPerpetuals) }) else null)
            }
            itemsPositioned(previewPerpetuals) { position, item ->
                PerpetualListItem(
                    item = item,
                    listPosition = position,
                    longPressState = longPressedPerpetual,
                    onTogglePin = { handleAction(WalletSearchAction.TogglePerpetualPin(it)) },
                    onClick = { handleAction(WalletSearchAction.OpenPerpetual(item.data.asset.toPrimitives())) },
                )
            }
        }
    } else {
        null
    }

    val listsContent: (LazyListScope.() -> Unit)? = if (lists.isNotEmpty()) {
        {
            item {
                SubheaderItem(R.string.common_lists)
            }
            itemsPositioned(lists) { position, item ->
                SearchListItem(
                    row = item,
                    listPosition = position,
                    onClick = { handleAction(WalletSearchAction.OpenList(item.list.id, item.list.name)) },
                )
            }
        }
    } else {
        null
    }

    val nftsContent: (LazyListScope.() -> Unit)? = if (previewNfts.isNotEmpty()) {
        {
            item {
                SubheaderItem(R.string.nft_collections, if (hasMoreNfts) ({ handleAction(WalletSearchAction.OpenCollections) }) else null)
            }
            itemsPositioned(previewNfts) { position, item ->
                NftListItem(
                    row = item.row,
                    listPosition = position,
                    onClick = {
                        when (val target = item.target) {
                            is NftItemTarget.Collection -> handleAction(WalletSearchAction.OpenNftCollection(target.id))
                            is NftItemTarget.Asset -> handleAction(WalletSearchAction.OpenNftAsset(target.id))
                        }
                    },
                )
            }
        }
    } else {
        null
    }

    SelectAssetScene(
        title = {
            SearchBar(
                query = viewModel.queryState,
                modifier = Modifier.listItem(ListPosition.Single, paddingHorizontal = space0),
                autoFocus = true,
            )
        },
        query = viewModel.queryState,
        pinned = pinned,
        popular = emptyList<AssetInfoDataAggregate>().toImmutableList(),
        unpinned = previewAssets.toImmutableList(),
        recent = recent,
        state = state,
        empty = searchEmptyState,
        searchable = false,
        onAction = { action ->
            when (action) {
                SelectAssetAction.Cancel -> handleAction(WalletSearchAction.Cancel)

                SelectAssetAction.AddAsset -> handleAction(WalletSearchAction.AddAsset)

                SelectAssetAction.OpenRecentsSheet -> handleAction(WalletSearchAction.OpenRecentsSheet)

                SelectAssetAction.ShowAllAssets -> handleAction(
                    WalletSearchAction.ShowAllAssets(viewModel.queryState.text.toString()),
                )

                is SelectAssetAction.Select -> handleAction(WalletSearchAction.OpenAsset(action.asset))

                is SelectAssetAction.SelectRecent -> handleAction(WalletSearchAction.OpenRecent(action.asset))

                is SelectAssetAction.ChainFilter,
                is SelectAssetAction.BalanceFilter,
                SelectAssetAction.ClearFilters,
                -> Unit
            }
        },
        recentsSheetEnabled = true,
        contextActions = AssetContextActions(
            onTogglePin = { handleAction(WalletSearchAction.PinAsset(it)) },
            onAddToWallet = { handleAction(WalletSearchAction.AddToWallet(it)) },
        ),
        pinnedPerpetualRows = pinnedPerpetualRows,
        perpetualsContent = perpetualsContent,
        listsContent = listsContent,
        nftsContent = nftsContent,
        assetsHeaderRes = R.string.assets_title,
        assetsHeaderClickable = hasMoreAssets,
        snackbar = snackbar,
    )

    RecentsScreen(viewModel = recentsViewModel, onSelect = { handleAction(WalletSearchAction.OpenRecent(it)) })
}

@Composable
private fun SearchListItem(row: GemSearchListRow, listPosition: ListPosition, onClick: () -> Unit) {
    ListItem(
        model = ListItemModel(title = row.list.name, subtitle = row.subtitle, image = ListItemImage.Url(row.imageUrl, placeholder = row.list.name)),
        listPosition = listPosition,
        modifier = Modifier.clickable(onClick = onClick),
        minHeight = ListItemDefaults.plainMinHeight,
        accessory = { DataBadgeChevron() },
    )
}
