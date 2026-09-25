package com.gemwallet.android.features.asset_select.presents.views

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import com.gemwallet.android.features.asset_select.viewmodels.AssetSelectViewModel
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemSelectAssetType

@Composable
fun SelectSendScreen(onCancel: () -> Unit, onSelect: ((AssetId) -> Unit)?, chains: List<Chain> = emptyList(), viewModel: AssetSelectViewModel = assetSelectViewModel(GemSelectAssetType.Send)) {
    LaunchedEffect(chains) {
        viewModel.setChainFilter(chains)
    }

    AssetSelectScreen(
        onSelect = onSelect,
        onSelectRecent = onSelect,
        onCancel = onCancel,
        viewModel = viewModel,
    )
}
