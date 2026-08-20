package com.gemwallet.android

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.data.repositories.bridge.ActiveWalletConnectRequest
import com.gemwallet.android.data.repositories.bridge.WalletConnectUserRequest
import com.gemwallet.android.features.bridge.views.AuthRequestScene
import com.gemwallet.android.features.bridge.views.ProposalScene
import com.gemwallet.android.features.bridge.views.RequestScene
import com.gemwallet.android.features.confirm.presents.AcquireAssetAction
import com.gemwallet.android.ui.R
import com.wallet.core.primitives.AssetId

@Composable
internal fun rememberWalletConnectOverlay(
    activeRequest: ActiveWalletConnectRequest,
    onError: (String) -> Unit,
): @Composable ((AcquireAssetAction, AssetId) -> Unit) -> Unit = remember(activeRequest, onError) {
    { onAcquireAsset ->
        WalletConnectOverlay(
            activeRequest = activeRequest,
            onAcquireAsset = onAcquireAsset,
            onError = onError,
        )
    }
}

@Composable
internal fun WalletConnectErrorDialog(
    error: String?,
    onDismiss: () -> Unit,
) {
    if (!error.isNullOrEmpty()) {
        AlertDialog(
            onDismissRequest = onDismiss,
            containerColor = MaterialTheme.colorScheme.background,
            confirmButton = {
                TextButton(onClick = onDismiss) {
                    Text(text = stringResource(id = R.string.common_done))
                }
            },
            text = {
                Text(text = error)
            }
        )
    }
}

@Composable
private fun WalletConnectOverlay(
    activeRequest: ActiveWalletConnectRequest,
    onAcquireAsset: (AcquireAssetAction, AssetId) -> Unit,
    onError: (String) -> Unit,
) {
    val request by activeRequest.current.collectAsStateWithLifecycle()

    Box(
        modifier = Modifier.navigationBarsPadding(),
    ) {
        when (val current = request) {
            null -> Unit
            is WalletConnectUserRequest.AuthenticationRequest -> AuthRequestScene(
                request = current.request,
                verifyContext = current.verifyContext,
            )
            is WalletConnectUserRequest.SessionProposal -> ProposalScene(
                proposal = current.proposal,
                verifyContext = current.verifyContext,
                onError = onError,
            )
            is WalletConnectUserRequest.SessionRequest -> RequestScene(
                request = current.request,
                verifyContext = current.verifyContext,
                onAcquireAsset = onAcquireAsset,
                onError = onError,
            )
        }
    }
}
