package com.gemwallet.android.features.wallet_connector.presents

import androidx.activity.compose.BackHandler
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.wallet_connector.viewmodels.AuthRequestUIState
import com.gemwallet.android.features.wallet_connector.viewmodels.AuthRequestViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.gemGroupRows
import com.gemwallet.android.ui.components.screen.FatalStateScene
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.models.ButtonState
import com.wallet.core.primitives.ChainAddress
import uniffi.gemstone.GemRowAction
import uniffi.gemstone.GemWalletConnectAuthentication

@Composable
fun AuthRequestScreen(authentication: GemWalletConnectAuthentication, onOpenAddress: (ChainAddress) -> Unit, onError: (String) -> Unit) {
    val viewModel: AuthRequestViewModel = hiltViewModel()
    BackHandler(onBack = viewModel::onReject)
    val state by viewModel.state.collectAsStateWithLifecycle()
    val buttonState by viewModel.buttonState.collectAsStateWithLifecycle()

    LaunchedEffect(authentication.id) {
        viewModel.onRequest(authentication, onError)
    }

    when (val currentState = state) {
        is AuthRequestUIState.Error -> FatalStateScene(
            title = stringResource(id = R.string.wallet_connect_connect_title),
            message = currentState.text,
            onCancel = viewModel::onReject,
        )

        AuthRequestUIState.Loading -> LoadingScene(
            title = stringResource(id = R.string.transfer_review_request),
            onCancel = viewModel::onReject,
            closeIcon = true,
        )

        is AuthRequestUIState.Content -> AuthRequestContent(
            state = currentState,
            buttonState = buttonState,
            onWalletSelected = viewModel::onWalletSelected,
        ) { action ->
            when (action) {
                WalletConnectReviewAction.Approve -> viewModel.onApprove()
                WalletConnectReviewAction.Reject -> viewModel.onReject()
                is WalletConnectReviewAction.OpenAddress -> onOpenAddress(action.address)
            }
        }
    }
}

@Composable
private fun AuthRequestContent(state: AuthRequestUIState.Content, buttonState: ButtonState, onWalletSelected: (com.wallet.core.primitives.WalletId) -> Unit, onAction: (WalletConnectReviewAction) -> Unit) {
    var isShowSelectWallets by rememberSaveable { mutableStateOf(false) }

    WalletConnectReviewScene(
        model = state,
        buttonState = buttonState,
        details = {
            gemGroupRows(
                groups = state.sections,
                onSelect = if (state is AuthRequestUIState.Approving) null else ({ action -> if (action == GemRowAction.SelectWallet) isShowSelectWallets = true }),
            )
        },
        onAction = onAction,
    )

    SelectWalletSheet(
        isVisible = isShowSelectWallets,
        walletSections = state.availableWalletSections,
        selectedWalletId = state.selectedWallet.id,
        onWalletSelected = onWalletSelected,
        onDismissRequest = { isShowSelectWallets = false },
    )
}
