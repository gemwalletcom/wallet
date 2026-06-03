package com.gemwallet.android

import android.widget.Toast
import android.widget.Toast.makeText
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.BasicAlertDialog
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.bridge.views.AuthRequestScene
import com.gemwallet.android.features.bridge.views.ProposalScene
import com.gemwallet.android.features.bridge.views.RequestScene
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.models.actions.AssetIdAction
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall

@Composable
internal fun rememberWalletConnectOverlay(
    viewModel: WalletConnectViewModel,
    onError: (String) -> Unit,
): @Composable (AssetIdAction) -> Unit = remember(viewModel, onError) {
    { onBuy -> WalletConnectOverlay(viewModel = viewModel, onBuy = onBuy, onError = onError) }
}

@Composable
internal fun WalletConnectPairingToast(
    visible: Boolean,
    onShown: () -> Unit,
) {
    val context = LocalContext.current
    val message = stringResource(id = R.string.wallet_connect_connection_title)
    LaunchedEffect(visible) {
        if (visible) {
            makeText(context, message, Toast.LENGTH_SHORT).show()
            onShown()
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun WalletConnectErrorDialog(
    error: String?,
    onDismiss: () -> Unit,
) {
    if (!error.isNullOrEmpty()) {
        BasicAlertDialog(
            onDismissRequest = onDismiss,
        ) {
            Box(
                contentAlignment = Alignment.Center,
                modifier = Modifier.background(
                    MaterialTheme.colorScheme.background,
                    shape = RoundedCornerShape(paddingSmall)
                )
            ) {
                Column(
                    modifier = Modifier.padding(start = paddingDefault, end = paddingDefault, top = paddingDefault),
                    horizontalAlignment = Alignment.End
                ) {
                    Text(
                        modifier = Modifier.fillMaxWidth(),
                        text = error,
                        style = MaterialTheme.typography.titleMedium,
                        fontWeight = FontWeight.W400,
                        textAlign = TextAlign.Center,
                    )
                    Spacer16()
                    TextButton(onClick = onDismiss) {
                        Text(text = stringResource(id = R.string.common_cancel))
                    }
                }
            }
        }
    }
}

@Composable
private fun WalletConnectOverlay(
    viewModel: WalletConnectViewModel,
    onBuy: AssetIdAction,
    onError: (String) -> Unit,
) {
    val context = LocalContext.current
    val walletConnect by viewModel.uiState.collectAsStateWithLifecycle()

    LaunchedEffect(walletConnect) {
        when (val event = walletConnect) {
            is WalletConnectIntent.SessionProposal -> {
                if (event.verifyContext == null) {
                    makeText(context, "Session Proposal Error: Verify Context is not available", Toast.LENGTH_LONG).show()
                    viewModel.rejectSessionProposal(event.sessionProposal)
                }
            }
            is WalletConnectIntent.SessionRequest -> {
                if (event.verifyContext == null) {
                    viewModel.rejectSessionRequest(event.request)
                }
            }
            is WalletConnectIntent.AuthRequest -> {
                if (event.verifyContext == null) {
                    viewModel.rejectSessionAuthenticate(event.request)
                }
            }
            is WalletConnectIntent.ConnectionState,
            WalletConnectIntent.Idle,
            WalletConnectIntent.Cancel -> Unit
        }
    }

    Box(
        modifier = Modifier.navigationBarsPadding(),
    ) {
        when (val event = walletConnect) {
            is WalletConnectIntent.ConnectionState,
            WalletConnectIntent.Idle,
            WalletConnectIntent.Cancel -> Unit
            is WalletConnectIntent.AuthRequest -> {
                event.verifyContext?.let { verifyContext ->
                    AuthRequestScene(
                        request = event.request,
                        verifyContext = verifyContext,
                        onCancel = viewModel::onCancel,
                    )
                }
            }
            is WalletConnectIntent.SessionProposal -> {
                event.verifyContext?.let { verifyContext ->
                    ProposalScene(
                        proposal = event.sessionProposal,
                        verifyContext = verifyContext,
                        onCancel = viewModel::onCancel,
                        onError = onError,
                    )
                }
            }
            is WalletConnectIntent.SessionRequest -> {
                event.verifyContext?.let { verifyContext ->
                    RequestScene(
                        request = event.request,
                        verifyContext = verifyContext,
                        onBuy = onBuy,
                        onCancel = viewModel::onCancel,
                    )
                }
            }
        }
    }
}
