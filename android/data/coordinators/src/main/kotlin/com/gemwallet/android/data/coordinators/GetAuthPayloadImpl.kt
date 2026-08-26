package com.gemwallet.android.data.coordinators

import com.gemwallet.android.application.GetAuthPayload
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.application.device.coordinators.GetDeviceId
import com.gemwallet.android.blockchain.services.GemSignAuthOperator
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.referralChain
import com.wallet.core.primitives.AuthPayload
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet
import com.gemwallet.android.serializer.toJson
import uniffi.gemstone.createAuthMessage
import java.io.IOException
import uniffi.gemstone.GemAuthService
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.AuthNonce

class GetAuthPayloadImpl(
    private val authService: GemAuthService,
    private val getDeviceId: GetDeviceId,
    private val passwordStore: PasswordStore,
    private val signAuthOperator: GemSignAuthOperator,
) : GetAuthPayload {

    override suspend fun getAuthPayload(wallet: Wallet): AuthPayload {
        val chain = Chain.referralChain
        val account = wallet.getAccount(chain) ?: throw Exception() // TODO
        val deviceId = getDeviceId.getDeviceId()

        val nonceJson = authService.getNonce()
        val nonce = nonceJson.decodeJson<AuthNonce>()
        val message = createAuthMessage(
            address = account.address,
            authNonce = nonceJson
        )

        val signature = signAuthOperator(wallet, chain, message.hash, passwordStore.getPassword(wallet.id.id))
        return AuthPayload(
            deviceId = deviceId,
            chain = account.chain,
            address = account.address,
            nonce = nonce.nonce,
            signature = signature
        )
    }
}
