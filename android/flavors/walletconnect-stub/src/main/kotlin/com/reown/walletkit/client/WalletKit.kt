package com.reown.walletkit.client

object WalletKit {
    private fun unavailable() = Wallet.Model.Error(UnsupportedOperationException("WalletConnect is not available in this build"))
    private fun fail(onError: (Wallet.Model.Error) -> Unit) = onError(unavailable())

    interface WalletDelegate {
        val onSessionAuthenticate: (Wallet.Model.SessionAuthenticate, Wallet.Model.VerifyContext) -> Unit
        fun onProposalExpired(proposal: Wallet.Model.ExpiredProposal)
        fun onRequestExpired(request: Wallet.Model.ExpiredRequest)
        fun onSessionDelete(sessionDelete: Wallet.Model.SessionDelete)
        fun onSessionExtend(session: Wallet.Model.Session)
        fun onSessionProposal(sessionProposal: Wallet.Model.SessionProposal, verifyContext: Wallet.Model.VerifyContext)
        fun onSessionRequest(sessionRequest: Wallet.Model.SessionRequest, verifyContext: Wallet.Model.VerifyContext)
        fun onSessionSettleResponse(settleSessionResponse: Wallet.Model.SettledSessionResponse)
        fun onSessionUpdateResponse(sessionUpdateResponse: Wallet.Model.SessionUpdateResponse)
    }

    fun initialize(
        params: Wallet.Params.Init,
        onSuccess: () -> Unit,
        onError: (Wallet.Model.Error) -> Unit,
    ) = onSuccess()

    fun setWalletDelegate(delegate: WalletDelegate) = Unit

    fun respondSessionRequest(
        params: Wallet.Params.SessionRequestResponse,
        onSuccess: () -> Unit,
        onError: (Wallet.Model.Error) -> Unit,
    ) = onSuccess()

    fun getPendingListOfSessionRequests(topic: String): List<Wallet.Model.SessionRequest> = emptyList()
    fun getVerifyContext(id: Long): Wallet.Model.VerifyContext? = null
    fun pingSession(params: Wallet.Params.Ping, onError: ((Wallet.Model.Error) -> Unit)?) = Unit
    fun getListOfActiveSessions(): List<Wallet.Model.Session> = emptyList()

    fun disconnectSession(
        params: Wallet.Params.SessionDisconnect,
        onSuccess: () -> Unit,
        onError: (Wallet.Model.Error) -> Unit,
    ) = onSuccess()

    fun pair(
        params: Wallet.Params.Pair,
        onSuccess: () -> Unit,
        onError: (Wallet.Model.Error) -> Unit,
    ) = fail(onError)

    fun generateApprovedNamespaces(
        sessionProposal: Wallet.Model.SessionProposal,
        supportedNamespaces: Map<String, Wallet.Model.Namespace.Session>,
    ): Map<String, Wallet.Model.Namespace.Session> = supportedNamespaces

    fun approveSession(
        params: Wallet.Params.SessionApprove,
        onError: (Wallet.Model.Error) -> Unit,
        onSuccess: () -> Unit,
    ) = fail(onError)

    fun rejectSession(
        params: Wallet.Params.SessionReject,
        onSuccess: () -> Unit,
        onError: (Wallet.Model.Error) -> Unit,
    ) = onSuccess()

    fun approveSessionAuthenticate(
        params: Wallet.Params.ApproveSessionAuthenticate,
        onSuccess: () -> Unit,
        onError: (Wallet.Model.Error) -> Unit,
    ) = fail(onError)

    fun rejectSessionAuthenticate(
        params: Wallet.Params.RejectSessionAuthenticate,
        onSuccess: () -> Unit,
        onError: (Wallet.Model.Error) -> Unit,
    ) = onSuccess()

    fun getSessionProposals(): List<Wallet.Model.SessionProposal> = emptyList()

    fun generateAuthPayloadParams(
        payloadParams: Wallet.Model.PayloadAuthRequestParams,
        supportedChains: List<String>,
        supportedMethods: List<String>,
    ) = Wallet.Model.PayloadAuthRequestParams(
        chains = payloadParams.chains.filter { it in supportedChains },
        methods = supportedMethods,
    )

    fun formatAuthMessage(params: Wallet.Params.FormatAuthMessage) = ""

    fun generateAuthObject(
        payloadParams: Wallet.Model.PayloadAuthRequestParams,
        issuer: String,
        signature: Wallet.Model.Cacao.Signature,
    ) = Wallet.Model.Cacao()
}
