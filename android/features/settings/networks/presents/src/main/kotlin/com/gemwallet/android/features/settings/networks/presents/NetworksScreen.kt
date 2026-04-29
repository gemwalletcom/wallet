package com.gemwallet.android.features.settings.networks.presents

import androidx.activity.compose.BackHandler
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.AnimatedContentTransitionScope
import androidx.compose.animation.core.tween
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.settings.networks.viewmodels.NetworksViewModel

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
            if (targetState != NetworksScreenState.Chains) {
                slideIntoContainer(
                    towards = AnimatedContentTransitionScope.SlideDirection.Left,
                    animationSpec = tween(350)
                ) togetherWith slideOutOfContainer(
                    towards = AnimatedContentTransitionScope.SlideDirection.Left,
                    animationSpec = tween(350)
                )
            } else {
                slideIntoContainer(
                    towards = AnimatedContentTransitionScope.SlideDirection.Right,
                    animationSpec = tween(350)
                ) togetherWith slideOutOfContainer(
                    towards = AnimatedContentTransitionScope.SlideDirection.Right,
                    animationSpec = tween(350)
                )
            }
        },
        label = "networks"
    ) { target ->
        when (target) {
            NetworksScreenState.Chains -> NetworksListScene(
                chains = state.chains,
                listState = selectListState,
                chainFilter = viewModel.chainFilter,
                onStatus = { showStatus = true },
                onSelect = viewModel::onSelectedChain,
                onCancel = onCancel
            )
            NetworksScreenState.Network -> NetworkScene(
                state = state,
                onRefresh = { viewModel.refresh() },
                onSelectNode = viewModel::onSelectNode,
                onDeleteNode = viewModel::onDeleteNode,
                onSelectBlockExplorer = viewModel::onSelectBlockExplorer,
                onCancel = viewModel::onSelectChain
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
