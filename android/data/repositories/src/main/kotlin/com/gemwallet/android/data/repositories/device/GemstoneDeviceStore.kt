package com.gemwallet.android.data.repositories.device

import com.gemwallet.android.data.service.store.ConfigStore
import uniffi.gemstone.GemDeviceStore

class GemstoneDeviceStore(
    private val configStore: ConfigStore,
) : GemDeviceStore {

    override suspend fun isRegistered(): Boolean = configStore.getBoolean(KEY_REGISTERED)

    override suspend fun setRegistered(registered: Boolean) = configStore.putBoolean(KEY_REGISTERED, registered)

    override suspend fun getSubscriptionsVersion(): Int = configStore.getInt(KEY_SUBSCRIPTION_VERSION)

    override suspend fun setSubscriptionsVersion(version: Int) = configStore.putInt(KEY_SUBSCRIPTION_VERSION, version)

    override suspend fun getPushedDevice(): String? = configStore.getString(KEY_PUSHED_DEVICE).takeIf { it.isNotEmpty() }

    override suspend fun setPushedDevice(device: String) = configStore.putString(KEY_PUSHED_DEVICE, device)

    override suspend fun getPushedSubscriptions(): String? = configStore.getString(KEY_PUSHED_SUBSCRIPTIONS).takeIf { it.isNotEmpty() }

    override suspend fun setPushedSubscriptions(signature: String) = configStore.putString(KEY_PUSHED_SUBSCRIPTIONS, signature)

    private companion object {
        const val KEY_REGISTERED = "device_registered"
        const val KEY_SUBSCRIPTION_VERSION = "subscription_version"
        const val KEY_PUSHED_DEVICE = "pushed_device"
        const val KEY_PUSHED_SUBSCRIPTIONS = "pushed_subscriptions"
    }
}
