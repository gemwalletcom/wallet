package com.gemwallet.android

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.application.wallet_connect.ActiveWalletConnectRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectUserRequest
import com.gemwallet.android.features.bridge.views.AuthRequestScene
import com.gemwallet.android.features.bridge.views.ProposalScene
import com.gemwallet.android.features.bridge.views.RequestScene
import com.gemwallet.android.features.confirm.presents.AcquireAssetAction
import com.wallet.core.primitives.AssetId

@Composable
internal fun WalletConnectRequestContent(
    activeRequest: ActiveWalletConnectRequest,
    requestKey: String,
    onAcquireAsset: (AcquireAssetAction, AssetId) -> Unit,
    onError: (String) -> Unit,
) {
    val request by activeRequest.current.collectAsStateWithLifecycle()
    when (val current = request?.takeIf { it.key == requestKey }) {
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
