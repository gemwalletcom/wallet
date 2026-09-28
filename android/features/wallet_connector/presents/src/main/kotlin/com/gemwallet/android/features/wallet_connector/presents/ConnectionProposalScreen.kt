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
import com.gemwallet.android.application.wallet_connect.WalletConnectSessionProposal
import com.gemwallet.android.application.wallet_connect.WalletConnectVerifyContext
import com.gemwallet.android.features.wallet_connector.viewmodels.ConnectionProposalViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.localization.text

@Composable
fun ConnectionProposalScreen(proposal: WalletConnectSessionProposal, verifyContext: WalletConnectVerifyContext, onError: (String) -> Unit) {
    val context = LocalContext.current
    val viewModel: ConnectionProposalViewModel = hiltViewModel()
    BackHandler(onBack = viewModel::onReject)
    val state by viewModel.state.collectAsStateWithLifecycle()
    val proposalRows by viewModel.proposalRows.collectAsStateWithLifecycle()
    val statusListItem by viewModel.statusListItem.collectAsStateWithLifecycle()
    val peer by viewModel.proposal.collectAsStateWithLifecycle()
    val selectedWallet by viewModel.selectedWallet.collectAsStateWithLifecycle()
    val availableWallets by viewModel.availableWallets.collectAsStateWithLifecycle()
    val availableWalletSections by viewModel.availableWalletSections.collectAsStateWithLifecycle()
    val buttonState by viewModel.buttonState.collectAsStateWithLifecycle()

    LaunchedEffect(proposal) {
        viewModel.onProposal(proposal, verifyContext) { message ->
            Toast.makeText(context, message, Toast.LENGTH_LONG).show()
        }
    }

    when (val currentPeer = peer) {
        null -> LoadingScene(
            title = stringResource(id = R.string.wallet_connect_connect_title),
            onCancel = viewModel::onReject,
            closeIcon = true,
        )

        else -> ConnectionProposalScene(
            peer = currentPeer,
            state = state,
            proposalRows = proposalRows,
            statusListItem = statusListItem,
            selectedWallet = selectedWallet,
            availableWallets = availableWallets,
            availableWalletSections = availableWalletSections,
            buttonState = buttonState,
            onReject = viewModel::onReject,
            onApprove = { viewModel.onApprove { error -> onError(error.text(context)) } },
            onWalletSelected = viewModel::onWalletSelected,
        )
    }
}
