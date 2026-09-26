package com.gemwallet.android.features.wallet_connector.presents

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.application.wallet_connect.WalletConnectPendingRequest
import com.gemwallet.android.features.wallet_connector.viewmodels.SignMessageViewModel
import com.wallet.core.primitives.ChainAddress

@Composable
fun SignMessageScreen(request: WalletConnectPendingRequest.SignMessage, onSigned: (String) -> Unit, onReject: () -> Unit, onOpenAddress: (ChainAddress) -> Unit, onError: (String) -> Unit) {
    val viewModel = hiltViewModel<SignMessageViewModel, SignMessageViewModel.Factory> { it.create(request) }
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()
    val buttonState by viewModel.buttonState.collectAsStateWithLifecycle()

    SignMessageScene(
        state = uiState,
        buttonState = buttonState,
        onSign = { viewModel.onSign(onSigned = onSigned, onError = onError) },
        onReject = { viewModel.onReject(onReject) },
        onOpenAddress = onOpenAddress,
    )
}
