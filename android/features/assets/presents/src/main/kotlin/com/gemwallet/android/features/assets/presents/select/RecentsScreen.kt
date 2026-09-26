package com.gemwallet.android.features.assets.presents.select

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.assets.viewmodels.select.RecentsViewModel
import com.wallet.core.primitives.Asset

@Composable
fun RecentsScreen(viewModel: RecentsViewModel, onSelect: (Asset) -> Unit) {
    val isVisible by viewModel.visible.collectAsStateWithLifecycle()
    val viewState by viewModel.viewState.collectAsStateWithLifecycle()
    var pendingAsset by remember { mutableStateOf<Asset?>(null) }

    LaunchedEffect(isVisible) {
        if (!isVisible) {
            pendingAsset?.let { asset ->
                pendingAsset = null
                onSelect(asset)
            }
        }
    }

    RecentsScene(
        isVisible = isVisible,
        viewState = viewState,
        query = viewModel.query,
        onDismissRequest = viewModel::dismiss,
        onClear = viewModel::onClear,
        onSelect = { asset ->
            pendingAsset = asset
            viewModel.dismiss()
        },
    )
}
