package com.gemwallet.android.features.settings.presents.chain_settings

import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.settings.viewmodels.chain_settings.AddNodeViewModel
import com.wallet.core.primitives.Chain

@Composable
fun AddNodeScreen(chain: Chain, onCancel: () -> Unit) {
    val viewModel: AddNodeViewModel = hiltViewModel()
    val state by viewModel.viewState.collectAsStateWithLifecycle()

    DisposableEffect(chain) {
        viewModel.init(chain)
        onDispose { }
    }

    AddNodeScene(
        chain = chain,
        state = state,
        url = viewModel.url,
        onUrlChange = viewModel::onUrlChange,
        onAdd = { viewModel.addUrl(onAdded = onCancel) },
        onCancel = onCancel,
    )
}
