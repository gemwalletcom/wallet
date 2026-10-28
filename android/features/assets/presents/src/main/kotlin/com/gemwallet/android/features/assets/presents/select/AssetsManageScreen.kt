package com.gemwallet.android.features.assets.presents.select

import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.assets.viewmodels.select.SelectAssetViewModel
import com.gemwallet.android.ui.icons.AppIcons
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemSelectAssetType

@Composable
fun AssetsManageScreen(onAddAsset: () -> Unit, onAssetClick: (AssetId) -> Unit, onCancel: () -> Unit, chain: Chain? = null, viewModel: SelectAssetViewModel = selectAssetViewModel(GemSelectAssetType.Manage, listOfNotNull(chain))) {
    val isAddAssetAvailable by viewModel.isAddAssetAvailable.collectAsStateWithLifecycle()

    SelectAssetScreen(
        onCancel = onCancel,
        onAddAsset = onAddAsset,
        actions = {
            if (isAddAssetAvailable) {
                IconButton(modifier = Modifier.testTag("add_custom_token"), onClick = onAddAsset) {
                    Icon(imageVector = AppIcons.Add, contentDescription = "")
                }
            }
        },
        viewModel = viewModel,
    )
}
