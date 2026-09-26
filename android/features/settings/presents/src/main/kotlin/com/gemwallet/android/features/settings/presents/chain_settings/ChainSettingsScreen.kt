package com.gemwallet.android.features.settings.presents.chain_settings

import androidx.activity.compose.BackHandler
import androidx.compose.animation.AnimatedContent
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.settings.viewmodels.chain_settings.ChainSettingsViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.animation.navigationSlideTransition
import com.gemwallet.android.ui.components.screen.rememberSnackbarState

@Composable
fun ChainSettingsScreen(onCancel: () -> Unit, viewModel: ChainSettingsViewModel = hiltViewModel()) {
    val state by viewModel.uiState.collectAsStateWithLifecycle()
    val snackbar = rememberSnackbarState(message = state.errorText, iconRes = R.drawable.ic_error, onShown = viewModel::clearError)

    val selectListState = rememberLazyListState()
    var showStatus by remember { mutableStateOf(false) }
    val screenState = when {
        showStatus -> ChainSettingsScreenState.Status
        state.selectChain -> ChainSettingsScreenState.Chains
        else -> ChainSettingsScreenState.Network
    }

    BackHandler(screenState != ChainSettingsScreenState.Chains) {
        when (screenState) {
            ChainSettingsScreenState.Status -> showStatus = false
            ChainSettingsScreenState.Network -> viewModel.onSelectChain()
            ChainSettingsScreenState.Chains -> Unit
        }
    }

    AnimatedContent(
        targetState = screenState,
        transitionSpec = {
            navigationSlideTransition(forward = targetState != ChainSettingsScreenState.Chains)
        },
        label = "networks",
    ) { target ->
        when (target) {
            ChainSettingsScreenState.Chains -> ChainListSettingsScene(
                chains = state.chains,
                listState = selectListState,
                chainFilter = viewModel.chainFilter,
                onAction = { action ->
                    when (action) {
                        ChainListSettingsAction.ShowStatus -> showStatus = true
                        is ChainListSettingsAction.Select -> viewModel.onSelectedChain(action.chain)
                        ChainListSettingsAction.Cancel -> onCancel()
                    }
                },
            )

            ChainSettingsScreenState.Network -> ChainSettingsScene(
                state = state,
                snackbar = snackbar,
                onAction = { action ->
                    when (action) {
                        ChainSettingsAction.Refresh -> viewModel.refresh()
                        ChainSettingsAction.Cancel -> viewModel.onSelectChain()
                        is ChainSettingsAction.SelectNode -> viewModel.onSelectNode(action.url)
                        is ChainSettingsAction.DeleteNode -> viewModel.onDeleteNode(action.url)
                        is ChainSettingsAction.SelectBlockExplorer -> viewModel.onSelectBlockExplorer(action.name)
                    }
                },
            )

            ChainSettingsScreenState.Status -> ServiceStatusScreen(
                onCancel = { showStatus = false },
            )
        }
    }
}

private enum class ChainSettingsScreenState {
    Chains,
    Network,
    Status,
}
