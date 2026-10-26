package com.gemwallet.android.features.settings.presents.chain_settings

import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import com.gemwallet.android.features.settings.viewmodels.chain_settings.ChainListSettingsViewModel
import com.wallet.core.primitives.Chain

@Composable
fun ChainListSettingsScreen(onSelectChain: (Chain) -> Unit, onShowStatus: () -> Unit, onCancel: () -> Unit, viewModel: ChainListSettingsViewModel = hiltViewModel()) {
    val chainFilter = rememberTextFieldState()
    val query = chainFilter.text.toString()

    ChainListSettingsScene(
        chains = remember(query) { viewModel.chains(query) },
        chainFilter = chainFilter,
        onAction = { action ->
            when (action) {
                ChainListSettingsAction.ShowStatus -> onShowStatus()
                is ChainListSettingsAction.Select -> onSelectChain(action.chain)
                ChainListSettingsAction.Cancel -> onCancel()
            }
        },
    )
}
