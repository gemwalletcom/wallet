package com.gemwallet.android.data.coordinators.wallet_connect

import com.gemwallet.android.testkit.mockGemWalletConnectAuthMessage
import uniffi.gemstone.GemWalletConnectAuthMessage
import uniffi.gemstone.GemWalletConnectClientInterface
import uniffi.gemstone.GemWalletConnectFailure
import uniffi.gemstone.GemWalletConnectRejectionReason
import uniffi.gemstone.Wallet
import java.util.Collections

class WalletConnectClientMock(
    private val needsConnection: Boolean = false,
    private val pairing: suspend (String) -> Unit = {},
    private val receiving: suspend (String) -> Unit = {},
    private val answering: suspend (String, String) -> GemWalletConnectFailure? = { _, _ -> null },
) : GemWalletConnectClientInterface {
    val calls: MutableList<String> = Collections.synchronizedList(mutableListOf())

    override fun needsConnection(): Boolean = needsConnection

    override suspend fun connected() {
        calls.add("connected")
    }

    override suspend fun restore() {
        calls.add("restore")
    }

    override suspend fun receive(frame: String) {
        calls.add("receive $frame")
        receiving(frame)
    }

    override suspend fun pair(uri: String) {
        calls.add("pair $uri")
        pairing(uri)
    }

    override suspend fun approveSession(proposalId: String, wallet: Wallet) {
        calls.add("approveSession $proposalId ${wallet.id}")
    }

    override suspend fun rejectSession(proposalId: String, reason: GemWalletConnectRejectionReason) {
        calls.add("rejectSession $proposalId $reason")
    }

    override suspend fun answerRequest(topic: String, requestId: String): GemWalletConnectFailure? {
        calls.add("answerRequest $topic $requestId")
        return answering(topic, requestId)
    }

    override suspend fun rejectRequest(topic: String, requestId: String) {
        calls.add("rejectRequest $topic $requestId")
    }

    override suspend fun disconnect(topic: String) {
        calls.add("disconnect $topic")
    }

    override fun authenticationMessage(authenticationId: String, wallet: Wallet): GemWalletConnectAuthMessage {
        calls.add("authenticationMessage $authenticationId ${wallet.id}")
        return mockGemWalletConnectAuthMessage()
    }

    override suspend fun approveAuthentication(authenticationId: String, wallet: Wallet, signature: String) {
        calls.add("approveAuthentication $authenticationId ${wallet.id} $signature")
    }

    override suspend fun rejectAuthentication(authenticationId: String) {
        calls.add("rejectAuthentication $authenticationId")
    }
}
