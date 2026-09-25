package com.gemwallet.android.features.asset_select.presents.views

import androidx.compose.runtime.Composable
import com.gemwallet.android.features.asset_select.viewmodels.AssetSelectViewModel
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemSelectAssetType

@Composable
fun SelectReceiveScreen(
    onCancel: () -> Unit,
    onSelect: ((AssetId) -> Unit)?,
    titleContent: (@Composable () -> Unit)? = null,
    closeIcon: Boolean = false,
    showFilter: Boolean? = null,
    viewModel: AssetSelectViewModel = assetSelectViewModel(GemSelectAssetType.Receive),
) {
    AssetSelectScreen(
        titleContent = titleContent,
        closeIcon = closeIcon,
        showFilter = showFilter,
        onSelectRecent = onSelect,
        onCancel = onCancel,
        onSelect = onSelect,
        viewModel = viewModel,
    )
}
