package com.gemwallet.android.data.coordinators.wallet_connect

import com.gemwallet.android.application.wallet_connect.coordinators.PrepareSessionProposal
import com.gemwallet.android.application.wallet_connect.values.WalletConnectPairingProposal
import com.gemwallet.android.serializer.decodeJson
import uniffi.gemstone.GemWalletConnectServiceInterface
import uniffi.gemstone.WalletConnectionVerificationStatus

class PrepareSessionProposalImpl(
    private val walletConnectService: GemWalletConnectServiceInterface,
) : PrepareSessionProposal {

    override suspend fun invoke(
        name: String,
        description: String,
        url: String,
        icons: List<String>,
        requiredChainIds: List<String>,
        optionalChainIds: List<String>,
        origin: String?,
        validation: WalletConnectionVerificationStatus,
    ): WalletConnectPairingProposal {
        val prepared = walletConnectService.prepareSessionProposal(
            requiredChainIds = requiredChainIds,
            optionalChainIds = optionalChainIds,
            metadata = walletConnectService.applicationMetadata(name, description, url, icons),
            origin = origin,
            validation = validation,
        )
        return WalletConnectPairingProposal(
            proposal = prepared.proposal.decodeJson(),
            verificationStatus = prepared.verificationStatus,
        )
    }
}
