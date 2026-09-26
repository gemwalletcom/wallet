package com.gemwallet.android.features.wallet_connector.presents

import android.widget.Toast
import androidx.activity.compose.BackHandler
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
import com.gemwallet.android.application.wallet_connect.WalletConnectAuthenticationRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectVerifyContext
import com.gemwallet.android.features.wallet_connector.viewmodels.AuthRequestUIState
import com.gemwallet.android.features.wallet_connector.viewmodels.AuthRequestViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyNetworkItem
import com.gemwallet.android.ui.components.screen.FatalStateScene
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.ListPosition
import com.wallet.core.primitives.ChainAddress

@Composable
fun AuthRequestScreen(request: WalletConnectAuthenticationRequest, verifyContext: WalletConnectVerifyContext, onOpenAddress: (ChainAddress) -> Unit) {
    val context = LocalContext.current
    val viewModel: AuthRequestViewModel = hiltViewModel()
    BackHandler(onBack = viewModel::onReject)
    val state by viewModel.state.collectAsStateWithLifecycle()
    val buttonState by viewModel.buttonState.collectAsStateWithLifecycle()

    LaunchedEffect(request.id) {
        viewModel.onRequest(request, verifyContext) { message ->
            Toast.makeText(context, message, Toast.LENGTH_LONG).show()
        }
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
            onApprove = viewModel::onApprove,
            onReject = viewModel::onReject,
            onWalletSelected = viewModel::onWalletSelected,
            onOpenAddress = onOpenAddress,
        )
    }
}

@Composable
private fun AuthRequestContent(
    state: AuthRequestUIState.Content,
    buttonState: ButtonState,
    onApprove: () -> Unit,
    onReject: () -> Unit,
    onWalletSelected: (com.wallet.core.primitives.WalletId) -> Unit,
    onOpenAddress: (ChainAddress) -> Unit,
) {
    var isShowSelectWallets by remember { mutableStateOf(false) }
    val canSelectWallet = state.canChooseWallet

    WalletConnectReviewScene(
        model = state,
        buttonState = buttonState,
        details = {
            val hasHeader = state.header != null
            if (hasHeader) {
                item { ListItem(model = state.appListItem, listPosition = ListPosition.First) }
            }
            item {
                ListItem(
                    model = state.walletListItem,
                    listPosition = if (hasHeader) ListPosition.Middle else ListPosition.First,
                    modifier = if (canSelectWallet && state !is AuthRequestUIState.Approving) {
                        Modifier.clickable { isShowSelectWallets = true }
                    } else {
                        Modifier
                    },
                    accessory = if (canSelectWallet) {
                        { DataBadgeChevron() }
                    } else {
                        null
                    },
                )
            }
            item { PropertyNetworkItem(state.chain, listPosition = ListPosition.Last) }
        },
        onApprove = onApprove,
        onReject = onReject,
        onOpenAddress = onOpenAddress,
    )

    SelectWalletSheet(
        isVisible = isShowSelectWallets,
        walletSections = state.availableWalletSections,
        selectedWalletId = state.selectedWallet.id,
        onWalletSelected = onWalletSelected,
        onDismissRequest = { isShowSelectWallets = false },
    )
}
