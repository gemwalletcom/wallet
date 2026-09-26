package com.gemwallet.android.features.swap.presents

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.domains.swap.SwapItemType
import com.gemwallet.android.features.assets.presents.select.SelectAssetScreen
import com.gemwallet.android.features.assets.viewmodels.select.RecentsViewModel
import com.gemwallet.android.features.swap.viewmodels.SwapSelectViewModel
import com.wallet.core.primitives.AssetId

@Composable
fun SwapSelectScreen(onCancel: () -> Unit, onSelect: (select: SwapItemType, assetId: AssetId) -> Unit, viewModel: SwapSelectViewModel = hiltViewModel(), recentsViewModel: RecentsViewModel = hiltViewModel()) {
    val select by viewModel.select.collectAsStateWithLifecycle()

    val onSelectAsset: (AssetId) -> Unit = { assetId -> onSelect(select, assetId) }

    SelectAssetScreen(
        onCancel = onCancel,
        onSelect = onSelectAsset,
        onSelectRecent = onSelectAsset,
        viewModel = viewModel,
        recentsViewModel = recentsViewModel,
    )
}
