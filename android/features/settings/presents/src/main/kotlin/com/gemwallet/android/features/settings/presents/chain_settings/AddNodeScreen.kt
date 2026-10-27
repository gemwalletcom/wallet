package com.gemwallet.android.features.settings.presents.chain_settings

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.settings.viewmodels.chain_settings.AddNodeViewModel
import com.wallet.core.primitives.Chain

@Composable
fun AddNodeScreen(chain: Chain, onCancel: () -> Unit) {
    val viewModel = hiltViewModel<AddNodeViewModel, AddNodeViewModel.Factory> { it.create(chain) }
    val state by viewModel.viewState.collectAsStateWithLifecycle()
    val isAdded by viewModel.isAdded.collectAsStateWithLifecycle()
    val close = {
        viewModel.clear()
        onCancel()
    }

    LaunchedEffect(isAdded) {
        if (isAdded) {
            viewModel.isAdded.value = false
            close()
        }
    }

    AddNodeScene(
        chain = chain,
        state = state,
        url = viewModel.url,
        onUrlChange = viewModel::onUrlChange,
        onAdd = viewModel::addUrl,
        onCancel = close,
    )
}
