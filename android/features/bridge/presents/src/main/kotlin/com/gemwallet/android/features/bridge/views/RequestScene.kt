package com.gemwallet.android.features.bridge.views

import android.widget.Toast
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.data.repositories.bridge.WalletConnectSessionRequest
import com.gemwallet.android.data.repositories.bridge.WalletConnectVerifyContext
import com.gemwallet.android.features.bridge.viewmodels.RequestSceneState
import com.gemwallet.android.features.bridge.viewmodels.WCRequestViewModel
import com.gemwallet.android.features.bridge.viewmodels.model.BridgeRequestError
import com.gemwallet.android.features.bridge.viewmodels.model.WCRequest
import com.gemwallet.android.features.confirm.presents.ConfirmScreen
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.features.confirm.presents.AcquireAssetAction
import com.wallet.core.primitives.AssetId

@Composable
fun RequestScene(
    request: WalletConnectSessionRequest,
    verifyContext: WalletConnectVerifyContext,
    onAcquireAsset: (AcquireAssetAction, AssetId) -> Unit,
    onCancel: () -> Unit,
    onError: (String) -> Unit,
) {
    val viewModel: WCRequestViewModel = hiltViewModel()
    val context = LocalContext.current

    DisposableEffect(request.topic, request.request.id) {
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
                    else -> Unit
                }
            },
            onError = onError,
        )

        onDispose { viewModel.reset() }
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
                    walletRow = { PropertyItem(R.string.common_wallet, sceneState.walletName, listPosition = ListPosition.First) },
                    onApprove = { viewModel.onSign(onError) },
                    onReject = viewModel::onReject,
                )
                is WCRequest.Transaction -> ConfirmScreen(
                    params = request.confirmParams,
                    simulationResult = request.simulation,
                    finishAction = { hash -> viewModel.onTransactionResult(hash, onError) },
                    onAcquireAsset = onAcquireAsset,
                    cancelAction = viewModel::onReject,
                    handleSystemBack = true,
                )
            }
        }
        RequestSceneState.Cancel -> onCancel()
    }
}
