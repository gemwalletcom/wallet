package com.gemwallet.android.features.bridge.views

import android.widget.Toast
import androidx.compose.foundation.clickable
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.bridge.viewmodels.AuthSceneState
import com.gemwallet.android.features.bridge.viewmodels.WCAuthViewModel
import com.gemwallet.android.features.bridge.viewmodels.model.BridgeRequestError
import com.gemwallet.android.data.repositories.bridge.WalletConnectAuthenticationRequest
import com.gemwallet.android.data.repositories.bridge.WalletConnectVerifyContext
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.components.screen.FatalStateScene
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.models.ListPosition

@Composable
fun AuthRequestScene(
    request: WalletConnectAuthenticationRequest,
    verifyContext: WalletConnectVerifyContext,
) {
    val context = LocalContext.current
    val viewModel: WCAuthViewModel = hiltViewModel()
    val state by viewModel.state.collectAsStateWithLifecycle()
    val buttonState by viewModel.buttonState.collectAsStateWithLifecycle()

    LaunchedEffect(request.id) {
        viewModel.onRequest(request, verifyContext) { error ->
            when (error) {
                BridgeRequestError.MaliciousSession -> Toast.makeText(
                    context,
                    R.string.errors_connections_malicious_origin,
                    Toast.LENGTH_LONG
                ).show()
            }
        }
    }

    when (val currentState = state) {
        is AuthSceneState.Error -> FatalStateScene(
            title = stringResource(id = R.string.wallet_connect_connect_title),
            message = currentState.cause?.walletConnectMessage()
                ?: currentState.message
                ?: stringResource(id = R.string.errors_unknown_try_again),
            onCancel = viewModel::onReject,
        )
        AuthSceneState.Loading -> LoadingScene(
            title = stringResource(id = R.string.transfer_review_request),
            onCancel = viewModel::onReject,
            closeIcon = true,
        )
        is AuthSceneState.Content -> AuthRequestContent(
            state = currentState,
            buttonState = buttonState,
            onApprove = viewModel::onApprove,
            onReject = viewModel::onReject,
            onWalletSelected = viewModel::onWalletSelected,
        )
    }
}

@Composable
private fun AuthRequestContent(
    state: AuthSceneState.Content,
    buttonState: ButtonState,
    onApprove: () -> Unit,
    onReject: () -> Unit,
    onWalletSelected: (com.wallet.core.primitives.WalletId) -> Unit,
) {
    var isShowSelectWallets by remember { mutableStateOf(false) }
    val canSelectWallet = state.availableWallets.size > 1

    WalletConnectReviewScene(
        model = state,
        buttonState = buttonState,
        walletRow = {
            PropertyItem(
                modifier = if (canSelectWallet && state !is AuthSceneState.Approving) {
                    Modifier.clickable { isShowSelectWallets = true }
                } else {
                    Modifier
                },
                title = { PropertyTitleText(R.string.common_wallet) },
                data = {
                    PropertyDataText(
                        text = state.selectedWallet.name,
                        badge = if (canSelectWallet) {
                            { DataBadgeChevron() }
                        } else {
                            null
                        },
                    )
                },
                listPosition = ListPosition.First,
            )
        },
        onApprove = onApprove,
        onReject = onReject,
    )

    WalletSelectionSheet(
        isVisible = isShowSelectWallets,
        wallets = state.availableWallets,
        selectedWalletId = state.selectedWallet.id,
        onWalletSelected = onWalletSelected,
        onDismissRequest = { isShowSelectWallets = false },
    )
}
