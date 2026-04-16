package com.gemwallet.android.features.asset_select.presents.views

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.asset_select.viewmodels.RecentsSheetViewModel
import com.wallet.core.primitives.AssetId

@Composable
fun RecentsSheetHost(
    viewModel: RecentsSheetViewModel,
    onSelect: (AssetId) -> Unit,
) {
    val isVisible by viewModel.visible.collectAsStateWithLifecycle()
    val uiModel by viewModel.uiModel.collectAsStateWithLifecycle()
    var pendingAssetId by remember { mutableStateOf<AssetId?>(null) }

    LaunchedEffect(isVisible) {
        if (!isVisible) {
            pendingAssetId?.let { id ->
                pendingAssetId = null
                onSelect(id)
            }
        }
    }

    RecentsBottomSheet(
        isVisible = isVisible,
        uiModel = uiModel,
        query = viewModel.query,
        onDismissRequest = viewModel::dismiss,
        onClear = viewModel::onClear,
        onSelect = { id ->
            pendingAssetId = id
            viewModel.dismiss()
        },
    )
}
