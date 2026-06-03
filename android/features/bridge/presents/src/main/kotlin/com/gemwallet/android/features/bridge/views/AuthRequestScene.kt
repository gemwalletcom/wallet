package com.gemwallet.android.features.bridge.views

import android.widget.Toast
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
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
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.list_head.CenteredListHead
import com.gemwallet.android.ui.components.list_head.CenteredListHeadSubtitleLayout
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyNetworkItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.components.screen.FatalStateScene
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.simulation.simulationPayloadFieldsContent
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.requestAuth
import com.gemwallet.android.ui.theme.paddingDefault
import com.reown.walletkit.client.Wallet

@Composable
fun AuthRequestScene(
    request: Wallet.Model.SessionAuthenticate,
    verifyContext: Wallet.Model.VerifyContext,
    onCancel: () -> Unit,
) {
    val context = LocalContext.current
    val viewModel: WCAuthViewModel = hiltViewModel()
    val state by viewModel.state.collectAsStateWithLifecycle()

    LaunchedEffect(request.id) {
        viewModel.onRequest(request, verifyContext)
    }

    when (val currentState = state) {
        AuthSceneState.Canceled -> {
            LaunchedEffect(currentState) {
                onCancel()
            }
        }
        AuthSceneState.ScamCanceled -> {
            val maliciousOriginMessage = stringResource(R.string.errors_connections_malicious_origin)
            LaunchedEffect(currentState) {
                Toast.makeText(
                    context,
                    maliciousOriginMessage,
                    Toast.LENGTH_LONG
                ).show()
                onCancel()
            }
        }
        is AuthSceneState.Error -> FatalStateScene(
            title = stringResource(id = R.string.wallet_connect_connect_title),
            message = currentState.message,
            onCancel = viewModel::onReject,
        )
        AuthSceneState.Loading -> LoadingScene(
            title = stringResource(id = R.string.transfer_review_request),
            onCancel = viewModel::onReject,
            closeIcon = true,
        )
        is AuthSceneState.Content -> AuthRequestContent(
            state = currentState,
            onApprove = viewModel::onApprove,
            onReject = viewModel::onReject,
            onWalletSelected = viewModel::onWalletSelected,
        )
    }
}

@Composable
private fun AuthRequestContent(
    state: AuthSceneState.Content,
    onApprove: () -> Unit,
    onReject: () -> Unit,
    onWalletSelected: (com.wallet.core.primitives.WalletId) -> Unit,
) {
    val context = LocalContext.current
    var isShowSelectWallets by remember { mutableStateOf(false) }
    var sheetType by remember { mutableStateOf<AuthRequestSheetType?>(null) }
    val canSelectWallet = state.availableWallets.size > 1

    Scene(
        title = stringResource(id = R.string.transfer_review_request),
        backHandle = true,
        closeIcon = true,
        mainAction = {
            MainActionButton(
                title = stringResource(id = R.string.transfer_confirm),
                loading = state is AuthSceneState.Approving,
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
                    icon = state.peer.icon,
                    title = state.peer.name,
                    subtitle = state.peer.uri,
                    contentDescription = "wallet_connect_app_icon",
                    subtitleLayout = CenteredListHeadSubtitleLayout.Vertical,
                )
            }
            item {
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
            }
            item {
                PropertyNetworkItem(state.approval.chain, listPosition = ListPosition.Last)
            }
            if (state.approval.hasPayload) {
                simulationPayloadFieldsContent(
                    fields = state.approval.primaryPayloadFields,
                    onDetailsClick = { sheetType = AuthRequestSheetType.Details },
                )
            } else {
                walletConnectTextMessage(state.approval.message)
            }
        }
    }

    when (sheetType) {
        AuthRequestSheetType.Details -> {
            WalletConnectPayloadDetailsSheet(
                primaryFields = state.approval.primaryPayloadFields,
                secondaryFields = state.approval.secondaryPayloadFields,
                onViewFullMessage = { sheetType = AuthRequestSheetType.FullMessage },
                onDismissRequest = { sheetType = null },
            )
        }
        AuthRequestSheetType.FullMessage -> {
            WalletConnectFullMessageSheet(
                message = state.approval.message,
                onDismissRequest = { sheetType = null },
            )
        }
        null -> Unit
    }

    WalletSelectionSheet(
        isVisible = isShowSelectWallets,
        wallets = state.availableWallets,
        selectedWalletId = state.selectedWallet.id,
        onWalletSelected = onWalletSelected,
        onDismissRequest = { isShowSelectWallets = false },
    )
}

private enum class AuthRequestSheetType {
    Details,
    FullMessage,
}
