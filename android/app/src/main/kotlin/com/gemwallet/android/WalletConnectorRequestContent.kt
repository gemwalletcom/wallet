package com.gemwallet.android

import android.widget.Toast
import androidx.activity.compose.BackHandler
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.application.wallet_connect.ActiveWalletConnectRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectSessionRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectUserRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectVerifyContext
import com.gemwallet.android.features.transfer.presents.confirm.ConfirmTransferScreen
import com.gemwallet.android.features.transfer.viewmodels.confirm.models.GetAssetAction
import com.gemwallet.android.features.wallet_connector.presents.AuthRequestScreen
import com.gemwallet.android.features.wallet_connector.presents.ConnectionProposalScreen
import com.gemwallet.android.features.wallet_connector.presents.SignMessageScreen
import com.gemwallet.android.features.wallet_connector.viewmodels.WalletConnectorRequestViewModel
import com.gemwallet.android.features.wallet_connector.viewmodels.models.WalletConnectorRequestUIState
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.models.actions.FinishConfirmAction
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.ChainAddress

@Composable
internal fun WalletConnectorRequestContent(activeRequest: ActiveWalletConnectRequest, requestKey: String, onGetAsset: (GetAssetAction, AssetId) -> Unit, onOpenAddress: (ChainAddress) -> Unit, onError: (String) -> Unit) {
    val request by activeRequest.current.collectAsStateWithLifecycle()
    when (val current = request?.takeIf { it.key == requestKey }) {
        null -> Unit

        is WalletConnectUserRequest.AuthenticationRequest -> AuthRequestScreen(
            request = current.request,
            verifyContext = current.verifyContext,
            onOpenAddress = onOpenAddress,
        )

        is WalletConnectUserRequest.SessionProposal -> ConnectionProposalScreen(
            proposal = current.proposal,
            verifyContext = current.verifyContext,
            onError = onError,
        )

        is WalletConnectUserRequest.SessionRequest -> SessionRequestContent(
            request = current.request,
            verifyContext = current.verifyContext,
            onGetAsset = onGetAsset,
            onOpenAddress = onOpenAddress,
            onError = onError,
        )
    }
}

@Composable
private fun SessionRequestContent(
    request: WalletConnectSessionRequest,
    verifyContext: WalletConnectVerifyContext,
    onGetAsset: (GetAssetAction, AssetId) -> Unit,
    onOpenAddress: (ChainAddress) -> Unit,
    onError: (String) -> Unit,
    viewModel: WalletConnectorRequestViewModel = hiltViewModel(),
) {
    BackHandler(onBack = viewModel::onReject)
    val context = LocalContext.current

    LaunchedEffect(request.topic, request.request.id) {
        viewModel.onRequest(
            sessionRequest = request,
            verifyContext = verifyContext,
            onNotify = { message ->
                Toast.makeText(context, message, Toast.LENGTH_LONG).show()
            },
            onError = onError,
        )
    }

    val uiState by viewModel.uiState.collectAsStateWithLifecycle()

    when (val state = uiState) {
        WalletConnectorRequestUIState.Loading -> LoadingScene(
            title = stringResource(id = R.string.transfer_review_request),
            onCancel = viewModel::onReject,
            closeIcon = true,
        )

        is WalletConnectorRequestUIState.SignMessage -> SignMessageScreen(
            request = state.request,
            onSigned = viewModel::onResult,
            onReject = viewModel::onReject,
            onOpenAddress = onOpenAddress,
            onError = onError,
        )

        is WalletConnectorRequestUIState.Transaction -> ConfirmTransferScreen(
            input = state.input,
            simulationResult = state.simulation,
            finishAction = FinishConfirmAction { hash, _ -> viewModel.onResult(hash) },
            onGetAsset = onGetAsset,
            onOpenAddress = onOpenAddress,
            cancelAction = CancelAction(viewModel::onReject),
            handleSystemBack = true,
        )
    }
}
