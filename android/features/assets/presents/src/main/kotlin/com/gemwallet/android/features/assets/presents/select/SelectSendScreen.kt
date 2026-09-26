package com.gemwallet.android.features.assets.presents.select

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import com.gemwallet.android.features.assets.viewmodels.select.AssetSelectViewModel
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
