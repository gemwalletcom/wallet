package com.gemwallet.android.data.coordinators.wallet_connect

import com.gemwallet.android.application.wallet_connect.coordinators.PrepareSessionProposal
import com.gemwallet.android.application.wallet_connect.values.WalletConnectPairingProposal
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import kotlinx.coroutines.flow.firstOrNull
import uniffi.gemstone.GemWalletConnectServiceInterface
import uniffi.gemstone.WalletConnectionVerificationStatus

class PrepareSessionProposalImpl(
    private val sessionRepository: SessionRepository,
    private val walletsRepository: WalletsRepository,
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
        val wallets = walletsRepository.getAll().firstOrNull().orEmpty()
        val currentWalletId = sessionRepository.session().value?.wallet?.id
        val prepared = walletConnectService.prepareSessionProposal(
            wallets = wallets.map { it.toJson() },
            currentWalletId = currentWalletId?.id,
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
