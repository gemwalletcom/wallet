package com.gemwallet.android.features.bridge.views

import android.widget.Toast
import androidx.activity.compose.BackHandler
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.application.wallet_connect.WalletConnectSessionRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectVerifyContext
import com.gemwallet.android.features.bridge.viewmodels.RequestSceneState
import com.gemwallet.android.features.bridge.viewmodels.WCRequestViewModel
import com.gemwallet.android.features.bridge.viewmodels.model.BridgeRequestError
import com.gemwallet.android.features.bridge.viewmodels.model.WCRequest
import com.gemwallet.android.features.confirm.presents.ConfirmScreen
import com.gemwallet.android.features.confirm.viewmodels.models.AcquireAssetAction
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.wallet.core.primitives.AssetId

@Composable
fun RequestScene(
    request: WalletConnectSessionRequest,
    verifyContext: WalletConnectVerifyContext,
    onAcquireAsset: (AcquireAssetAction, AssetId) -> Unit,
    onError: (String) -> Unit,
) {
    val viewModel: WCRequestViewModel = hiltViewModel()
    BackHandler(onBack = viewModel::onReject)
    val context = LocalContext.current
    val unknownErrorMessage = stringResource(id = R.string.errors_unknown_try_again)
    val reportError: (String) -> Unit = { message -> onError(message.ifBlank { unknownErrorMessage }) }

    LaunchedEffect(request.topic, request.request.id) {
        viewModel.onRequest(
            sessionRequest = request,
            verifyContext = verifyContext,
            onNotify = { error ->
                when (error) {
                    BridgeRequestError.MaliciousSession -> Toast.makeText(
                        context,
                        R.string.errors_connections_malicious_origin,
                        Toast.LENGTH_LONG
                    ).show()
                    BridgeRequestError.Expired -> Toast.makeText(
                        context,
                        R.string.wallet_connect_request_expired,
                        Toast.LENGTH_LONG
                    ).show()
                }
            },
            onError = reportError,
        )
    }

    val sceneState by viewModel.sceneState.collectAsStateWithLifecycle()
    val buttonState by viewModel.buttonState.collectAsStateWithLifecycle()

    when (sceneState) {
        RequestSceneState.Loading -> LoadingScene(
            title = stringResource(id = R.string.transfer_review_request),
            onCancel = viewModel::onReject,
            closeIcon = true,
        )
        is RequestSceneState.Content -> (sceneState as RequestSceneState.Content).let { sceneState ->
            val request = sceneState.request
            when (request) {
                is WCRequest.SignMessage -> WalletConnectReviewScene(
                    model = request,
                    buttonState = buttonState,
                    walletRow = { position -> ListItem(model = sceneState.walletListItem, listPosition = position) },
                    onApprove = { viewModel.onSign(reportError) },
                    onReject = viewModel::onReject,
                )
                is WCRequest.Transaction -> ConfirmScreen(
                    input = request.input,
                    simulationResult = request.simulation,
                    finishAction = { hash, _ -> viewModel.onTransactionResult(hash) },
                    onAcquireAsset = onAcquireAsset,
                    cancelAction = viewModel::onReject,
                    handleSystemBack = true,
                )
            }
        }
    }
}
