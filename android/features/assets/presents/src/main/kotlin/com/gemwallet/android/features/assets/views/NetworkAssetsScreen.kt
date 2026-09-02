package com.gemwallet.android.features.assets.views

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.asset_select.presents.views.assetRows
import com.gemwallet.android.features.asset_select.presents.views.getAssetBadge
import com.gemwallet.android.features.assets.viewmodels.NetworkAssetsViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.empty.EmptyContentType
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.list_item.AssetContextActions
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.ui.components.list_item.PinnedAssetsHeaderItem
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.assetPriceSupport
import com.gemwallet.android.ui.components.list_item.getBalanceInfo
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.AssetsGroupType
import com.wallet.core.primitives.AssetId

@Composable
fun NetworkAssetsScreen(
    onSelectAsset: (AssetId) -> Unit,
    onManageAssets: () -> Unit,
    onCancel: () -> Unit,
    viewModel: NetworkAssetsViewModel = hiltViewModel(),
) {
    val pinned by viewModel.pinned.collectAsStateWithLifecycle()
    val unpinned by viewModel.unpinned.collectAsStateWithLifecycle()
    val hidden by viewModel.hidden.collectAsStateWithLifecycle()
    val isEmpty by viewModel.isEmpty.collectAsStateWithLifecycle()
    val longPressedAsset = remember { mutableStateOf<AssetId?>(null) }
    val activeActions = remember(viewModel) {
        AssetContextActions(onTogglePin = viewModel::togglePin, onHide = viewModel::hideAsset)
    }
    val hiddenActions = remember(viewModel) {
        AssetContextActions(onTogglePin = viewModel::togglePin, onAddToWallet = viewModel::addToWallet)
    }

    Scene(
        title = viewModel.title,
        onClose = onCancel,
        actions = {
            IconButton(onClick = onManageAssets) {
                Icon(imageVector = AppIcons.Tune, contentDescription = "")
            }
        },
    ) {
        LazyColumn(modifier = Modifier.fillMaxSize()) {
            if (pinned.isNotEmpty()) {
                item { PinnedAssetsHeaderItem(AssetsGroupType.Pinned) }
                networkAssetRows(pinned, onSelectAsset, longPressedAsset, activeActions)
            }
            networkAssetRows(unpinned, onSelectAsset, longPressedAsset, activeActions)
            if (hidden.isNotEmpty()) {
                item { SubheaderItem(R.string.common_hidden) }
                networkAssetRows(hidden, onSelectAsset, longPressedAsset, hiddenActions)
            }
            if (isEmpty) {
                item {
                    EmptyContentView(
                        type = EmptyContentType.NetworkAssets(onManageAssets = onManageAssets),
                        modifier = Modifier.fillParentMaxSize(),
                    )
                }
            }
        }
    }
}

private fun LazyListScope.networkAssetRows(
    items: List<AssetInfoDataAggregate>,
    onSelect: (AssetId) -> Unit,
    longPressedAsset: MutableState<AssetId?>,
    contextActions: AssetContextActions,
) {
    assetRows(
        items = items,
        onSelect = { onSelect(it.id) },
        support = { assetPriceSupport(it.price) },
        titleBadge = ::getAssetBadge,
        itemTrailing = { getBalanceInfo(it)() },
        longPressedAsset = longPressedAsset,
        contextActions = contextActions,
    )
}
