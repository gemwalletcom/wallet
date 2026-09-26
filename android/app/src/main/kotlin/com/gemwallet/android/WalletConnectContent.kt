package com.gemwallet.android

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.application.wallet_connect.ActiveWalletConnectRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectUserRequest
import com.gemwallet.android.features.transfer.presents.confirm.ConfirmTransferScreen
import com.gemwallet.android.features.transfer.viewmodels.confirm.models.GetAssetAction
import com.gemwallet.android.features.wallet_connector.presents.AuthRequestScene
import com.gemwallet.android.features.wallet_connector.presents.ProposalScene
import com.gemwallet.android.features.wallet_connector.presents.RequestScene
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.ChainAddress

@Composable
internal fun WalletConnectRequestContent(activeRequest: ActiveWalletConnectRequest, requestKey: String, onGetAsset: (GetAssetAction, AssetId) -> Unit, onOpenAddress: (ChainAddress) -> Unit, onError: (String) -> Unit) {
    val request by activeRequest.current.collectAsStateWithLifecycle()
    when (val current = request?.takeIf { it.key == requestKey }) {
        null -> Unit

        is WalletConnectUserRequest.AuthenticationRequest -> AuthRequestScene(
            request = current.request,
            verifyContext = current.verifyContext,
            onOpenAddress = onOpenAddress,
        )

        is WalletConnectUserRequest.SessionProposal -> ProposalScene(
            proposal = current.proposal,
            verifyContext = current.verifyContext,
            onError = onError,
        )

        is WalletConnectUserRequest.SessionRequest -> RequestScene(
            request = current.request,
            verifyContext = current.verifyContext,
            confirmContent = { input, simulation, finishAction, cancelAction ->
                ConfirmTransferScreen(
                    input = input,
                    simulationResult = simulation,
                    finishAction = finishAction,
                    onGetAsset = onGetAsset,
                    onOpenAddress = onOpenAddress,
                    cancelAction = cancelAction,
                    handleSystemBack = true,
                )
            },
            onOpenAddress = onOpenAddress,
            onError = onError,
        )
    }
}
