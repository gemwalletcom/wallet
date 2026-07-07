package com.gemwallet.android.features.assets.views

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.assets.viewmodels.HiddenAssetsViewModel
import com.gemwallet.android.features.assets.views.components.assets
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.AssetContextActions
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.AssetsGroupType
import com.wallet.core.primitives.AssetId

@Composable
fun HiddenAssetsScreen(
    onAction: (HiddenAssetsAction) -> Unit,
    viewModel: HiddenAssetsViewModel = hiltViewModel(),
) {
    val hiddenAssets by viewModel.hiddenAssets.collectAsStateWithLifecycle()
    val longPressedAsset = remember { mutableStateOf<AssetId?>(null) }
    val assetActions = remember(viewModel) {
        AssetContextActions(
            onTogglePin = viewModel::togglePin,
            onHide = viewModel::hideAsset,
        )
    }

    Scene(
        title = stringResource(R.string.asset_verification_unverified),
        onClose = { onAction(HiddenAssetsAction.Close) },
    ) {
        LazyColumn(
            modifier = Modifier.fillMaxSize(),
        ) {
            assets(
                items = hiddenAssets,
                longPressState = longPressedAsset,
                group = AssetsGroupType.None,
                onAssetClick = { onAction(HiddenAssetsAction.OpenAsset(it)) },
                actions = assetActions,
            )
        }
    }
}
