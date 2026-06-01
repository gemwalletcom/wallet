package com.gemwallet.android.data.coordinators.device

import com.gemwallet.android.application.SecurityStore
import com.gemwallet.android.application.device.coordinators.GetDeviceId
import com.gemwallet.android.data.services.gemapi.DeviceKeyPair
import kotlinx.coroutines.runBlocking

class GetDeviceIdImpl(
    private val store: SecurityStore<Any>
) : GetDeviceId {

    private val deviceId: String

    init {
        val (_, deviceId) = runBlocking { initDeviceId() }
        this.deviceId = deviceId
    }

    private suspend fun initDeviceId(): Pair<String, String> {
        try {
            val data = Pair(store.getValue(Keys.PrivateKey), store.getValue(Keys.PublicKey))
            return data
        } catch (_: Throwable) {}

        val deviceKey = DeviceKeyPair.generate()
        val privateKey = deviceKey.privateKeyHex
        val publicKey = deviceKey.publicKeyHex

        store.putValue(Keys.PrivateKey, privateKey)
        store.putValue(Keys.PublicKey, publicKey)

        return Pair(privateKey, publicKey)
    }

    override fun getDeviceId(): String = deviceId

    override fun getDeviceKey(): String {
        return runBlocking { store.getValue(Keys.PrivateKey) }
    }

    enum class Keys(private val keyValue: String) {
        PrivateKey("private_key"),
        PublicKey("public_key")
        ;

        override fun toString(): String = keyValue
    }
}
