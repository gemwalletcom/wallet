package com.gemwallet.android.data.coordinators.device

import com.gemwallet.android.application.SecurityStore
import com.gemwallet.android.application.device.coordinators.GetDeviceId
import com.gemwallet.android.math.hex
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import uniffi.gemstone.generateDeviceKeyPair

class GetDeviceIdImpl(
    private val store: SecurityStore<Any>
) : GetDeviceId {

    private val mutex = Mutex()
    @Volatile
    private var deviceKeys: DeviceKeys? = null

    override suspend fun getDeviceId(): String = getDeviceKeys().publicKey

    override suspend fun getDeviceKey(): String = getDeviceKeys().privateKey

    private suspend fun getDeviceKeys(): DeviceKeys {
        return deviceKeys ?: mutex.withLock {
            deviceKeys ?: initDeviceKeys().also { deviceKeys = it }
        }
    }

    private suspend fun initDeviceKeys(): DeviceKeys {
        try {
            return DeviceKeys(
                privateKey = store.getValue(Keys.PrivateKey),
                publicKey = store.getValue(Keys.PublicKey),
            )
        } catch (_: Throwable) {}

        val deviceKey = generateDeviceKeyPair()
        val privateKey = deviceKey.privateKey.hex
        val publicKey = deviceKey.publicKey.hex

        store.putValue(Keys.PrivateKey, privateKey)
        store.putValue(Keys.PublicKey, publicKey)

        return DeviceKeys(privateKey, publicKey)
    }

    private data class DeviceKeys(
        val privateKey: String,
        val publicKey: String,
    )

    enum class Keys(private val keyValue: String) {
        PrivateKey("private_key"),
        PublicKey("public_key")
        ;

        override fun toString(): String = keyValue
    }
}
