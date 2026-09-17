package com.gemwallet.android.features.asset_select.presents.views

import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.features.asset_select.viewmodels.ManageSelectViewModel
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain

@Composable
fun AssetsManageScreen(
    onAddAsset: () -> Unit,
    onAssetClick: (AssetId) -> Unit,
    onCancel: () -> Unit,
    chain: Chain? = null,
    viewModel: ManageSelectViewModel = hiltViewModel(),
) {
    LaunchedEffect(chain) {
        viewModel.setChainFilter(listOfNotNull(chain))
    }

    val isAddAssetAvailable by viewModel.isAddAssetAvailable.collectAsStateWithLifecycle()

    AssetSelectScreen(
        onCancel = onCancel,
        onAddAsset = onAddAsset,
        actions = {
            if (isAddAssetAvailable) {
                IconButton(onClick = onAddAsset) {
                    Icon(imageVector = AppIcons.Add, contentDescription = "")
                }
            }
        },
        viewModel = viewModel,
    )
}
