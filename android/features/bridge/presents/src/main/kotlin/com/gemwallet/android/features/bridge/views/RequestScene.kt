@file:OptIn(ExperimentalMaterial3Api::class)

package com.gemwallet.android.features.bridge.views

import android.widget.Toast
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
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
import com.gemwallet.android.features.bridge.viewmodels.RequestSceneState
import com.gemwallet.android.features.bridge.viewmodels.WCRequestViewModel
import com.gemwallet.android.features.bridge.viewmodels.model.BridgeRequestError
import com.gemwallet.android.features.bridge.viewmodels.model.WCRequest
import com.gemwallet.android.features.confirm.presents.ConfirmScreen
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.list_head.CenteredListHead
import com.gemwallet.android.ui.components.list_head.CenteredListHeadSubtitleLayout
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyNetworkItem
import com.gemwallet.android.ui.components.screen.FatalStateScene
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.simulation.simulationPayloadDetailsContent
import com.gemwallet.android.ui.components.simulation.simulationPayloadFieldsContent
import com.gemwallet.android.ui.components.simulation.simulationWarningsContent
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.actions.AssetIdAction
import com.gemwallet.android.ui.models.hasCriticalWarning
import com.gemwallet.android.ui.requestAuth
import com.gemwallet.android.ui.theme.paddingDefault
import com.reown.walletkit.client.Wallet

@Composable
fun RequestScene(
    request: Wallet.Model.SessionRequest,
    verifyContext: Wallet.Model.VerifyContext,
    onBuy: AssetIdAction,
    onCancel: () -> Unit,
) {
    val viewModel: WCRequestViewModel = hiltViewModel()
    val context = LocalContext.current

    DisposableEffect(request.topic, request.request.id) {
        viewModel.onRequest(request, verifyContext) { error ->
            when (error) {
                BridgeRequestError.ScamSession -> Toast.makeText(
                    context,
                    R.string.errors_connections_malicious_origin,
                    Toast.LENGTH_LONG
                ).show()
                else -> Unit
            }
        }

        onDispose { viewModel.reset() }
    }

    val sceneState by viewModel.sceneState.collectAsStateWithLifecycle()

    when (sceneState) {
        RequestSceneState.Loading -> LoadingScene(
            title = stringResource(id = R.string.transfer_review_request),
            onCancel = viewModel::onReject,
            closeIcon = true,
        )
        is RequestSceneState.Error -> FatalStateScene(
            title = stringResource(id = R.string.wallet_connect_title),
            message = (sceneState as RequestSceneState.Error).message.ifBlank {
                stringResource(id = R.string.errors_unknown_try_again)
            },
            onCancel = viewModel::onReject
        )
        is RequestSceneState.Request -> (sceneState as RequestSceneState.Request).let { sceneState ->
            val request = sceneState.request
            when (request) {
                is WCRequest.SignMessage -> SignMessageScene(
                    sceneState.walletName,
                    request,
                    viewModel::onSign,
                    viewModel::onReject
                )
                is WCRequest.Transaction -> ConfirmScreen(
                    params = request.confirmParams,
                    walletConnectSimulation = request.simulation,
                    finishAction = { _, hash, _ -> viewModel.onTransactionResult(hash) },
                    onBuy = onBuy,
                    cancelAction = viewModel::onReject
                )
            }
        }
        RequestSceneState.Cancel -> onCancel()
    }
}

@Composable
private fun SignMessageScene(
    walletName: String,
    request: WCRequest.SignMessage,
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
                enabled = !request.simulation.warnings.hasCriticalWarning(),
            ) {
                context.requestAuth(AuthRequest.Phrase) {
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
            item { PropertyItem(R.string.common_wallet, walletName, listPosition = ListPosition.First) }
            item { PropertyNetworkItem(request.chain, listPosition = ListPosition.Last) }
            simulationWarningsContent(request.simulation.warnings)
            if (request.hasPayload) {
                simulationPayloadFieldsContent(
                    fields = request.primaryPayloadFields,
                    onDetailsClick = { sheetType = SignMessageSheetType.Details },
                )
            }

            if (!request.hasPayload) {
                textMessage(request.plainMessage)
            }
        }

        when (sheetType) {
            SignMessageSheetType.Details -> {
                ModalBottomSheet(
                    onDismissRequest = { sheetType = null },
                    sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true),
                    title = stringResource(R.string.common_details),
                ) {
                    LazyColumn {
                        simulationPayloadDetailsContent(
                            primaryFields = request.primaryPayloadFields,
                            secondaryFields = request.secondaryPayloadFields,
                        )
                        item {
                            PropertyItem(
                                action = R.string.sign_message_view_full_message,
                                listPosition = ListPosition.Single,
                                onClick = { sheetType = SignMessageSheetType.FullMessage },
                            )
                        }
                    }
                }
            }

            SignMessageSheetType.FullMessage -> {
                ModalBottomSheet(
                    onDismissRequest = { sheetType = null },
                    sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true),
                    title = stringResource(R.string.sign_message_view_full_message),
                ) {
                    Box(
                        modifier = Modifier
                            .fillMaxSize()
                            .verticalScroll(rememberScrollState())
                            .padding(paddingDefault)
                    ) {
                        Text(
                            modifier = Modifier.fillMaxWidth(),
                            text = request.plainMessage
                        )
                    }
                }
            }

            null -> Unit
        }
    }
}

private fun LazyListScope.textMessage(message: String) {
    item {
        SubheaderItem(R.string.sign_message_message)
        Text(
            modifier = Modifier
                .fillMaxWidth()
                .listItem()
                .padding(paddingDefault),
            text = message,
        )
    }
}

private enum class SignMessageSheetType {
    Details,
    FullMessage,
}
