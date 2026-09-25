package com.gemwallet.android.features.swap.views

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.domains.swap.SwapItemType
import com.gemwallet.android.features.asset_select.presents.views.AssetSelectScreen
import com.gemwallet.android.features.asset_select.viewmodels.RecentsSheetViewModel
import com.gemwallet.android.features.swap.viewmodels.SwapSelectViewModel
import com.wallet.core.primitives.AssetId

@Composable
fun SwapSelectScreen(onCancel: () -> Unit, onSelect: (select: SwapItemType, assetId: AssetId) -> Unit, viewModel: SwapSelectViewModel = hiltViewModel(), recentsViewModel: RecentsSheetViewModel = hiltViewModel()) {
    val select by viewModel.select.collectAsStateWithLifecycle()

    val onSelectAsset: (AssetId) -> Unit = { assetId -> onSelect(select, assetId) }

    AssetSelectScreen(
        onCancel = onCancel,
        onSelect = onSelectAsset,
        onSelectRecent = onSelectAsset,
        viewModel = viewModel,
        recentsViewModel = recentsViewModel,
    )
}
