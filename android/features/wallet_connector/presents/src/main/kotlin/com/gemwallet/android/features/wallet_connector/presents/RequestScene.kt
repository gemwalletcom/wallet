package com.gemwallet.android.features.wallet_connector.presents

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
import com.gemwallet.android.domains.confirm.ConfirmTransferInput
import com.gemwallet.android.features.wallet_connector.viewmodels.RequestSceneState
import com.gemwallet.android.features.wallet_connector.viewmodels.WCRequestViewModel
import com.gemwallet.android.features.wallet_connector.viewmodels.models.WCRequest
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.models.actions.FinishConfirmAction
import com.wallet.core.primitives.ChainAddress
import uniffi.gemstone.SimulationResult

@Composable
fun RequestScene(
    request: WalletConnectSessionRequest,
    verifyContext: WalletConnectVerifyContext,
    confirmContent: @Composable (input: ConfirmTransferInput, simulation: SimulationResult, finishAction: FinishConfirmAction, cancelAction: CancelAction) -> Unit,
    onOpenAddress: (ChainAddress) -> Unit,
    onError: (String) -> Unit,
) {
    val viewModel: WCRequestViewModel = hiltViewModel()
    BackHandler(onBack = viewModel::onReject)
    val context = LocalContext.current
    val reportError: (String) -> Unit = { message -> onError(message) }

    LaunchedEffect(request.topic, request.request.id) {
        viewModel.onRequest(
            sessionRequest = request,
            verifyContext = verifyContext,
            onNotify = { message ->
                Toast.makeText(context, message, Toast.LENGTH_LONG).show()
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
                    details = { itemsPositioned(request.rows) { position, row -> GemListRowView(row = row, listPosition = position) } },
                    onApprove = { viewModel.onSign(reportError) },
                    onReject = viewModel::onReject,
                    onOpenAddress = onOpenAddress,
                )

                is WCRequest.Transaction -> confirmContent(
                    request.input,
                    request.simulation,
                    FinishConfirmAction { hash, _ -> viewModel.onTransactionResult(hash) },
                    CancelAction(viewModel::onReject),
                )
            }
        }
    }
}
