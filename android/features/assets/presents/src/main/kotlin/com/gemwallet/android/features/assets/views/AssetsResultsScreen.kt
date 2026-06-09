package com.gemwallet.android.features.assets.views

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.asset_select.presents.views.assetRows
import com.gemwallet.android.features.asset_select.presents.views.getAssetBadge
import com.gemwallet.android.features.assets.viewmodels.AssetsResultsViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.AssetContextActions
import com.gemwallet.android.ui.components.list_item.PinnedAssetsHeaderItem
import com.gemwallet.android.ui.components.list_item.assetPriceSupport
import com.gemwallet.android.ui.components.list_item.getBalanceInfo
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.AssetsGroupType
import com.wallet.core.primitives.AssetId

@Composable
fun AssetsResultsScreen(
    onAction: (WalletSearchAction) -> Unit,
    viewModel: AssetsResultsViewModel = hiltViewModel(),
) {
    val pinned by viewModel.pinned.collectAsStateWithLifecycle()
    val cappedAssets by viewModel.cappedAssets.collectAsStateWithLifecycle()
    val longPressedAsset = remember { mutableStateOf<AssetId?>(null) }
    val onAssetClick: (AssetId) -> Unit = { onAction(WalletSearchAction.OpenAsset(it)) }
    val contextActions = remember(viewModel) {
        AssetContextActions(
            onTogglePin = viewModel::onTogglePin,
            onAddToWallet = { id -> viewModel.onChangeVisibility(id, true) },
        )
    }

    Scene(
        title = stringResource(id = R.string.assets_title),
        onClose = { onAction(WalletSearchAction.Cancel) },
    ) {
        LazyColumn(modifier = Modifier.fillMaxWidth()) {
            if (pinned.isNotEmpty()) {
                item { PinnedAssetsHeaderItem(AssetsGroupType.Pined) }
                assetRows(
                    items = pinned,
                    onSelect = onAssetClick,
                    support = { assetPriceSupport(it.price) },
                    titleBadge = ::getAssetBadge,
                    itemTrailing = { getBalanceInfo(it)() },
                    longPressedAsset = longPressedAsset,
                    contextActions = contextActions,
                )
            }
            assetRows(
                items = cappedAssets,
                onSelect = onAssetClick,
                support = { assetPriceSupport(it.price) },
                titleBadge = ::getAssetBadge,
                itemTrailing = { getBalanceInfo(it)() },
                longPressedAsset = longPressedAsset,
                contextActions = contextActions,
            )
        }
    }
}
