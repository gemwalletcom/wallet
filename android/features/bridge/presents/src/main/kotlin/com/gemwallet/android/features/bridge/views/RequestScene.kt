package com.gemwallet.android.features.bridge.views

import android.widget.Toast
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
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
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.components.list_head.CenteredListHead
import com.gemwallet.android.ui.components.list_head.CenteredListHeadSubtitleLayout
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyNetworkItem
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.message.SignMessageFullMessageSheet
import com.gemwallet.android.ui.components.message.SignMessageSheetType
import com.gemwallet.android.ui.components.message.SignMessagePayloadDetailsSheet
import com.gemwallet.android.ui.components.message.signMessageText
import com.gemwallet.android.ui.components.simulation.simulationPayloadFieldsContent
import com.gemwallet.android.ui.components.simulation.simulationWarningsContent
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.requestAuth
import com.gemwallet.android.ui.theme.paddingDefault
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
                is WCRequest.SignMessage -> SignMessageScene(
                    state = sceneState,
                    request = request,
                    buttonState = buttonState,
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

@Composable
private fun SignMessageScene(
    state: RequestSceneState.Content,
    request: WCRequest.SignMessage,
    buttonState: ButtonState,
    onApprove: () -> Unit,
    onReject: () -> Unit,
) {
    val context = LocalContext.current
    var sheetType by remember { mutableStateOf<SignMessageSheetType?>(null) }

    Scene(
        title = stringResource(id = R.string.transfer_review_request),
        backHandle = true,
        closeIcon = true,
        mainAction = {
            MainActionButton(
                title = stringResource(id = R.string.transfer_confirm),
                state = buttonState,
            ) {
                context.requestAuth(AuthRequest.Confirmation) {
                    onApprove()
                }
            }
        },
        onClose = onReject,
    ) { paddingValues ->
        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(bottom = paddingValues.calculateBottomPadding() + paddingDefault),
        ) {
            item {
                CenteredListHead(
                    icon = request.icon,
                    title = request.name,
                    subtitle = request.uri,
                    contentDescription = "wallet_connect_app_icon",
                    subtitleLayout = CenteredListHeadSubtitleLayout.Vertical,
                )
            }
            item { PropertyItem(R.string.common_wallet, state.walletName, listPosition = ListPosition.First) }
            item { PropertyNetworkItem(request.chain, listPosition = ListPosition.Last) }
            simulationWarningsContent(request.simulation.warnings)
            if (request.hasPayload) {
                simulationPayloadFieldsContent(
                    fields = request.primaryPayloadFields,
                    onDetailsClick = { sheetType = SignMessageSheetType.Details },
                )
            }

            if (!request.hasPayload) {
                signMessageText(request.plainMessage)
            }
        }

        when (sheetType) {
            SignMessageSheetType.Details -> {
                SignMessagePayloadDetailsSheet(
                    primaryFields = request.primaryPayloadFields,
                    secondaryFields = request.secondaryPayloadFields,
                    onViewFullMessage = { sheetType = SignMessageSheetType.FullMessage },
                    onDismissRequest = { sheetType = null },
                )
            }

            SignMessageSheetType.FullMessage -> {
                SignMessageFullMessageSheet(
                    message = request.plainMessage,
                    onDismissRequest = { sheetType = null },
                )
            }

            null -> Unit
        }
    }
}

