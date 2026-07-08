package com.gemwallet.android.features.settings.networks.presents

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
import com.gemwallet.android.features.settings.networks.viewmodels.NetworksViewModel
import com.gemwallet.android.ui.components.animation.navigationSlideTransition

@Composable
fun NetworksScreen(
    onCancel: () -> Unit,
    viewModel: NetworksViewModel = hiltViewModel()
) {
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    val selectListState = rememberLazyListState()
    var showStatus by remember { mutableStateOf(false) }
    val screenState = when {
        showStatus -> NetworksScreenState.Status
        state.selectChain -> NetworksScreenState.Chains
        else -> NetworksScreenState.Network
    }

    BackHandler(screenState != NetworksScreenState.Chains) {
        when (screenState) {
            NetworksScreenState.Status -> showStatus = false
            NetworksScreenState.Network -> viewModel.onSelectChain()
            NetworksScreenState.Chains -> Unit
        }
    }

    AnimatedContent(
        targetState = screenState,
        transitionSpec = {
            navigationSlideTransition(forward = targetState != NetworksScreenState.Chains)
        },
        label = "networks"
    ) { target ->
        when (target) {
            NetworksScreenState.Chains -> NetworksListScene(
                chains = state.chains,
                listState = selectListState,
                chainFilter = viewModel.chainFilter,
                onAction = { action ->
                    when (action) {
                        NetworksListAction.ShowStatus -> showStatus = true
                        is NetworksListAction.Select -> viewModel.onSelectedChain(action.chain)
                        NetworksListAction.Cancel -> onCancel()
                    }
                },
            )
            NetworksScreenState.Network -> NetworkScene(
                state = state,
                onAction = { action ->
                    when (action) {
                        NetworkAction.Refresh -> viewModel.refresh()
                        NetworkAction.Cancel -> viewModel.onSelectChain()
                        is NetworkAction.SelectNode -> viewModel.onSelectNode(action.node)
                        is NetworkAction.DeleteNode -> viewModel.onDeleteNode(action.node)
                        is NetworkAction.SelectBlockExplorer -> viewModel.onSelectBlockExplorer(action.name)
                    }
                },
            )
            NetworksScreenState.Status -> ServiceStatusScene(
                onCancel = { showStatus = false },
            )
        }
    }
}

private enum class NetworksScreenState {
    Chains,
    Network,
    Status,
}
