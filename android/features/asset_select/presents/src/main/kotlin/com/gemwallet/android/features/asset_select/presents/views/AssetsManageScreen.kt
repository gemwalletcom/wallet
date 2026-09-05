package com.gemwallet.android.features.asset_select.presents.views

import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Switch
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.ui.R
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
        title = stringResource(id = R.string.wallet_manage_token_list),
        titleBadge = ::getAssetBadge,
        onCancel = onCancel,
        onAddAsset = onAddAsset,
        actions = {
            if (isAddAssetAvailable) {
                IconButton(onClick = onAddAsset) {
                    Icon(imageVector = AppIcons.Add, contentDescription = "")
                }
            }
        },
        itemTrailing = { asset ->
            Switch(
                checked = asset.balanceEnabled,
                onCheckedChange = { viewModel.onChangeVisibility(asset.asset.id, it) },
            )
        },
        viewModel = viewModel,
    )
}
