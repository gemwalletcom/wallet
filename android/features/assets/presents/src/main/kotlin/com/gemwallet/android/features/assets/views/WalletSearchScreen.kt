package com.gemwallet.android.features.assets.views

import androidx.compose.foundation.clickable
import uniffi.gemstone.GemAssetAction
import com.wallet.core.primitives.AssetType
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.domains.asset.getListIconUrl
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.asset_select.presents.views.AssetSelectAction
import com.gemwallet.android.features.asset_select.presents.views.AssetSelectScene
import com.gemwallet.android.features.asset_select.presents.views.RecentsSheetHost
import com.gemwallet.android.features.asset_select.presents.views.getAssetBadge
import com.gemwallet.android.features.asset_select.viewmodels.RecentsSheetViewModel
import com.gemwallet.android.features.assets.viewmodels.WalletSearchViewModel
import com.gemwallet.android.features.perpetual.views.components.PerpetualItem
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.SearchBar
import com.gemwallet.android.ui.components.image.AsyncImage
import com.gemwallet.android.ui.components.list_item.AssetContextActions
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemDefaults
import com.gemwallet.android.ui.components.list_item.NftListItem
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.assetPriceSupport
import com.gemwallet.android.ui.components.list_item.getBalanceInfo
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.AssetToastEffect
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.iconSize
import com.wallet.core.primitives.AssetList
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
    val previewPerpetuals by viewModel.previewPerpetuals.collectAsStateWithLifecycle()
    val hasMorePerpetuals by viewModel.hasMorePerpetuals.collectAsStateWithLifecycle()
    val pinnedPerpetuals by viewModel.pinnedPerpetuals.collectAsStateWithLifecycle()
    val previewNfts by viewModel.previewNfts.collectAsStateWithLifecycle()
    val hasMoreNfts by viewModel.hasMoreNfts.collectAsStateWithLifecycle()
    val lists by viewModel.lists.collectAsStateWithLifecycle()

    val longPressedPerpetual = remember { mutableStateOf<PerpetualId?>(null) }
    val snackbar = remember { SnackbarHostState() }
    AssetToastEffect(viewModel.toastEvents, snackbar)

    val handleAction: (WalletSearchAction) -> Unit = { action ->
        when (action) {
            is WalletSearchAction.PinAsset -> viewModel.onPinAsset(action.assetId)
            is WalletSearchAction.AddToWallet -> viewModel.onAddToWallet(action.assetId)
            is WalletSearchAction.TogglePerpetualPin -> viewModel.onTogglePerpetualPin(action.perpetualId)
            WalletSearchAction.OpenRecentsSheet -> recentsViewModel.show(filters = viewModel.assetFilters())
            is WalletSearchAction.OpenRecent -> onAction(
                if (action.asset.type == AssetType.PERPETUAL) {
                    WalletSearchAction.OpenPerpetual(action.asset)
                } else {
                    WalletSearchAction.OpenAsset(action.asset)
                }
            )
            is WalletSearchAction.OpenAsset -> {
                viewModel.updateRecent(action.asset, GemAssetAction.OPEN)
                onAction(action)
            }
            is WalletSearchAction.OpenPerpetual -> {
                viewModel.updateRecent(action.asset, GemAssetAction.OPEN)
                onAction(action)
            }
            WalletSearchAction.AddAsset,
            WalletSearchAction.Cancel,
            WalletSearchAction.OpenPerpetuals,
            WalletSearchAction.OpenCollections,
            is WalletSearchAction.OpenNftCollection,
            is WalletSearchAction.OpenNftAsset,
            is WalletSearchAction.OpenList,
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
                onClick = { handleAction(WalletSearchAction.OpenPerpetual(item.asset)) },
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
                    onClick = { handleAction(WalletSearchAction.OpenPerpetual(item.asset)) },
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
                    list = item,
                    listPosition = position,
                    onClick = { handleAction(WalletSearchAction.OpenList(item.id, item.name)) },
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
                    model = item,
                    listPosition = position,
                    onClick = {
                        val asset = item.asset
                        if (asset == null) {
                            handleAction(WalletSearchAction.OpenNftCollection(item.collection.id.toIdentifier()))
                        } else {
                            handleAction(WalletSearchAction.OpenNftAsset(asset.id))
                        }
                    },
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
                autoFocus = true,
            )
        },
        titleBadge = ::getAssetBadge,
        support = { assetPriceSupport(it.price) },
        query = viewModel.queryState,
        pinned = pinned,
        popular = emptyList<AssetInfoDataAggregate>().toImmutableList(),
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
                    WalletSearchAction.ShowAllAssets(viewModel.queryState.text.toString())
                )
                is AssetSelectAction.Select -> handleAction(WalletSearchAction.OpenAsset(action.asset))
                is AssetSelectAction.SelectRecent -> handleAction(WalletSearchAction.OpenRecent(action.asset))
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
        listsContent = listsContent,
        nftsContent = nftsContent,
        assetsHeaderRes = R.string.assets_title,
        assetsHeaderClickable = hasMoreAssets,
        snackbar = snackbar,
    )

    RecentsSheetHost(viewModel = recentsViewModel, onSelect = { handleAction(WalletSearchAction.OpenRecent(it)) })
}

@Composable
private fun SearchListItem(
    list: AssetList,
    listPosition: ListPosition,
    onClick: () -> Unit,
) {
    ListItem(
        modifier = Modifier.clickable(onClick = onClick),
        listPosition = listPosition,
        minHeight = ListItemDefaults.plainMinHeight,
        leading = {
            AsyncImage(
                model = getListIconUrl(list.id),
                size = iconSize,
                placeholderText = list.name,
            )
        },
        title = { Text(text = list.name) },
        trailing = {
            DataBadgeChevron {
                Text(
                    text = list.count.toString(),
                    style = MaterialTheme.typography.bodyLarge,
                    color = MaterialTheme.colorScheme.secondary,
                )
            }
        },
    )
}
