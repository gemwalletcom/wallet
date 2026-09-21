package com.gemwallet.android.features.asset_select.presents.views

import androidx.compose.runtime.Composable
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import com.gemwallet.android.features.asset_select.viewmodels.ReceiveSelectViewModel
import com.wallet.core.primitives.AssetId

@Composable
fun SelectReceiveScreen(onCancel: () -> Unit, onSelect: ((AssetId) -> Unit)?, titleContent: (@Composable () -> Unit)? = null, closeIcon: Boolean = false, showFilter: Boolean? = null, viewModel: ReceiveSelectViewModel = hiltViewModel()) {
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
