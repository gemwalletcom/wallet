package com.gemwallet.android.features.asset_select.presents.views

import androidx.compose.runtime.Composable
import com.wallet.core.primitives.Asset
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.asset_select.viewmodels.RecentsSheetViewModel

@Composable
fun RecentsSheetHost(
    viewModel: RecentsSheetViewModel,
    onSelect: (Asset) -> Unit,
) {
    val isVisible by viewModel.visible.collectAsStateWithLifecycle()
    val uiModel by viewModel.uiModel.collectAsStateWithLifecycle()
    var pendingAsset by remember { mutableStateOf<Asset?>(null) }

    LaunchedEffect(isVisible) {
        if (!isVisible) {
            pendingAsset?.let { asset ->
                pendingAsset = null
                onSelect(asset)
            }
        }
    }

    RecentsBottomSheet(
        isVisible = isVisible,
        uiModel = uiModel,
        query = viewModel.query,
        onDismissRequest = viewModel::dismiss,
        onClear = viewModel::onClear,
        onSelect = { asset ->
            pendingAsset = asset
            viewModel.dismiss()
        },
    )
}
