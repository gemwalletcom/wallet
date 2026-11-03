package com.gemwallet.android.features.settings.presents.chain_settings

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.settings.viewmodels.chain_settings.ChainSettingsViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.rememberSnackbarState

@Composable
fun ChainSettingsScreen(onCancel: () -> Unit, viewModel: ChainSettingsViewModel = hiltViewModel()) {
    val state by viewModel.uiState.collectAsStateWithLifecycle()
    val snackbar = rememberSnackbarState(message = state.errorText, iconRes = R.drawable.ic_error, onShown = viewModel::clearError)

    ChainSettingsScene(
        state = state,
        snackbar = snackbar,
        onAction = { action ->
            when (action) {
                ChainSettingsAction.Refresh -> viewModel.refresh()
                ChainSettingsAction.Cancel -> onCancel()
                is ChainSettingsAction.SelectNode -> viewModel.onSelectNode(action.url)
                is ChainSettingsAction.DeleteNode -> viewModel.onDeleteNode(action.url)
                is ChainSettingsAction.SelectBlockExplorer -> viewModel.onSelectBlockExplorer(action.name)
            }
        },
    )
}
