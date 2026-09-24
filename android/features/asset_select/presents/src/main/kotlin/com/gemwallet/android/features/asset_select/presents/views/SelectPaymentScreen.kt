package com.gemwallet.android.features.asset_select.presents.views

import androidx.compose.runtime.Composable
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.asset_select.viewmodels.AssetSelectViewModel
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
