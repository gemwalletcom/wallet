package com.gemwallet.android.features.assets.presents.select

import androidx.compose.runtime.Composable
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.assets.viewmodels.select.AssetSelectViewModel
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemSelectAssetType

@Composable
fun SelectPaymentScreen(assetIds: List<AssetId>, onCancel: () -> Unit, onSelect: (AssetId) -> Unit, viewModel: AssetSelectViewModel = assetSelectViewModel(GemSelectAssetType.Payment(assetIds.map { it.toIdentifier() }))) {
    AssetSelectScreen(
        onCancel = onCancel,
        onSelect = onSelect,
        onSelectRecent = onSelect,
        viewModel = viewModel,
    )
}
