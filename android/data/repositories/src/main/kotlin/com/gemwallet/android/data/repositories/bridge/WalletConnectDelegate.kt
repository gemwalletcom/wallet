package com.gemwallet.android.data.repositories.bridge

import android.util.Log
import com.gemwallet.android.data.repositories.BuildConfig
import com.reown.android.CoreClient
import com.reown.walletkit.client.Wallet
import com.reown.walletkit.client.WalletKit
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.asSharedFlow

private const val TAG = "WalletConnect"
private fun log(msg: String) { if (BuildConfig.DEBUG) Log.d(TAG, msg) }

object WalletConnectDelegate : WalletKit.WalletDelegate, CoreClient.CoreDelegate {

    private val _walletEvents: MutableSharedFlow<WalletConnectEvent> = MutableSharedFlow(
        replay = 1,
        extraBufferCapacity = 64,
        onBufferOverflow = BufferOverflow.DROP_OLDEST,
    )
    internal val walletEvents: SharedFlow<WalletConnectEvent> = _walletEvents.asSharedFlow()

    fun bind() {
        CoreClient.setDelegate(this)
        WalletKit.setWalletDelegate(this)
    }

    override val onSessionAuthenticate: (Wallet.Model.SessionAuthenticate, Wallet.Model.VerifyContext) -> Unit = { sessionAuth, verifyContext ->
        log("onSessionAuthenticate id=${sessionAuth.id} verify=${verifyContext.validation}")
        _walletEvents.tryEmit(WalletConnectEvent(sessionAuth, verifyContext))
    }

    override fun onConnectionStateChange(state: Wallet.Model.ConnectionState) {
        log("onConnectionStateChange available=${state.isAvailable}")
        _walletEvents.tryEmit(WalletConnectEvent(state, null))
    }

    override fun onError(error: Wallet.Model.Error) {
        log("onError ${error.throwable.stackTraceToString()}")
        _walletEvents.tryEmit(WalletConnectEvent(error, null))
    }

    override fun onProposalExpired(proposal: Wallet.Model.ExpiredProposal) {
        log("onProposalExpired")
    }

    override fun onRequestExpired(request: Wallet.Model.ExpiredRequest) {
        log("onRequestExpired topic=${request.topic} id=${request.id}")
    }

    override fun onSessionDelete(sessionDelete: Wallet.Model.SessionDelete) {
        log("onSessionDelete $sessionDelete")
        _walletEvents.tryEmit(WalletConnectEvent(sessionDelete, null))
    }

    override fun onSessionExtend(session: Wallet.Model.Session) {
        log("onSessionExtend topic=${session.topic} expiry=${session.expiry}")
    }

    override fun onSessionProposal(
        sessionProposal: Wallet.Model.SessionProposal,
        verifyContext: Wallet.Model.VerifyContext
    ) {
        log("onSessionProposal name=${sessionProposal.name} url=${sessionProposal.url} verify=${verifyContext.validation}")
        _walletEvents.tryEmit(WalletConnectEvent(sessionProposal, verifyContext))
    }

    override fun onSessionRequest(
        sessionRequest: Wallet.Model.SessionRequest,
        verifyContext: Wallet.Model.VerifyContext
    ) {
        log("onSessionRequest method=${sessionRequest.request.method} chain=${sessionRequest.chainId} topic=${sessionRequest.topic}")
        _walletEvents.tryEmit(WalletConnectEvent(sessionRequest, verifyContext))
    }

    override fun onSessionSettleResponse(settleSessionResponse: Wallet.Model.SettledSessionResponse) {
        log("onSessionSettleResponse $settleSessionResponse")
        _walletEvents.tryEmit(WalletConnectEvent(settleSessionResponse, null))
    }

    override fun onSessionUpdateResponse(sessionUpdateResponse: Wallet.Model.SessionUpdateResponse) {
        log("onSessionUpdateResponse $sessionUpdateResponse")
        _walletEvents.tryEmit(WalletConnectEvent(sessionUpdateResponse, null))
    }
}
