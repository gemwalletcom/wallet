package com.gemwallet.android.data.repositories.device

import android.content.Context
import android.icu.util.ULocale
import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.preferencesDataStore
import com.gemwallet.android.application.device.coordinators.GetDeviceId
import com.gemwallet.android.application.session.coordinators.GetCurrentCurrency
import com.gemwallet.android.cases.device.GetPushEnabled
import com.gemwallet.android.cases.device.GetPushToken
import com.gemwallet.android.cases.device.IsDeviceRegistered
import com.gemwallet.android.cases.device.RequestPushToken
import com.gemwallet.android.cases.device.SetPushToken
import com.gemwallet.android.cases.device.SwitchPushEnabled
import com.gemwallet.android.cases.device.SyncDevice
import com.gemwallet.android.data.repositories.pricealerts.PriceAlertRepository
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.data.service.store.ConfigStore
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.model
import com.gemwallet.android.ext.os
import com.gemwallet.android.model.NotificationsAvailable
import com.gemwallet.android.serializer.jsonEncoder
import com.wallet.core.primitives.AddressChains
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Device
import com.wallet.core.primitives.DeviceLocale
import com.wallet.core.primitives.Platform
import com.wallet.core.primitives.PlatformStore
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletSubscription
import com.wallet.core.primitives.WalletSubscriptionChains
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.launch
import java.util.Locale

class DeviceRepository(
    private val context: Context,
    private val gemDeviceApiClient: GemDeviceApiClient,
    private val configStore: ConfigStore,
    private val requestPushToken: RequestPushToken,
    private val platformStore: PlatformStore,
    private val notificationsAvailable: NotificationsAvailable,
    private val versionName: String,
    private val getDeviceId: GetDeviceId,
    private val priceAlertRepository: PriceAlertRepository,
    private val getCurrentCurrency: GetCurrentCurrency,
    private val walletsRepository: WalletsRepository,
    private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.IO),
) : SwitchPushEnabled,
    GetPushEnabled,
    GetPushToken,
    SetPushToken,
    SyncDevice,
    IsDeviceRegistered
{
    private val Context.dataStore by preferencesDataStore(name = "device_config")

    private val syncCoordinator = DeviceSyncCoordinator(scope)

    override suspend fun syncDevice() {
        if (!needsSynchronization()) {
            return
        }
        syncCoordinator.synchronize {
            if (needsSynchronization()) {
                reconcileDevice()
            }
        }
    }

    override suspend fun switchPushEnabled(enabled: Boolean) {
        context.dataStore.edit { preferences ->
            preferences[Key.PushEnabled] = enabled && notificationsAvailable
        }
        try {
            syncDevice()
        } catch (_: Throwable) {}
    }

    override fun getPushEnabled(): Flow<Boolean> = context.dataStore.data
        .map { preferences -> notificationsAvailable && preferences[Key.PushEnabled] == true }

    override fun setPushToken(token: String) {
        configStore.putString(ConfigKey.PushToken.string, if (notificationsAvailable) token else "")
    }

    override suspend fun getPushToken(): String {
        return if (getPushEnabled().firstOrNull() == true) {
            configStore.getString(ConfigKey.PushToken.string)
        } else {
            ""
        }
    }

    private suspend fun needsSynchronization(): Boolean {
        if (!isDeviceRegistered()) {
            return true
        }
        val pushedDevice = pushedDevice() ?: return true
        val localDevice = buildDevice(
            pushToken = getPushToken(),
            pushEnabled = getPushEnabled().firstOrNull() ?: false,
            subscriptionsVersion = getSubscriptionVersion(),
        )

        return localDevice.hasChanges(pushedDevice)
            || loadWallets().subscriptionSignature() != pushedSubscriptionSignature()
    }

    private suspend fun reconcileDevice() {
        val wallets = loadWallets()
        val pushState = resolvePushState() ?: return
        val signature = wallets.subscriptionSignature()
        var version = getSubscriptionVersion()
        val localDevice = buildDevice(
            pushToken = pushState.token,
            pushEnabled = pushState.enabled,
            subscriptionsVersion = version,
        )
        val remoteDevice = getOrCreateDevice(localDevice)

        val signatureChanged = signature != pushedSubscriptionSignature()
        if (signatureChanged || remoteDevice.subscriptionsVersion != version) {
            reconcileSubscriptions(wallets)
            if (signatureChanged) {
                version += 1
                setSubscriptionVersion(version)
            }
        }

        val requestDevice = localDevice.copy(subscriptionsVersion = version)
        if (remoteDevice.hasChanges(requestDevice)) {
            gemDeviceApiClient.updateDevice(request = requestDevice)
            setDeviceRegistered(true)
        }
        recordPushedState(requestDevice, signature)
    }

    private suspend fun loadWallets(): List<Wallet> {
        return walletsRepository.getAll().firstOrNull() ?: emptyList()
    }

    override suspend fun isDeviceRegistered(): Boolean {
        return context.dataStore.data.map { it[Key.DeviceRegistered] }.firstOrNull() == true
    }

    private suspend fun getOrCreateDevice(device: Device): Device {
        if (isDeviceRegistered() || gemDeviceApiClient.isDeviceRegistered()) {
            gemDeviceApiClient.getDevice()?.let { remoteDevice ->
                setDeviceRegistered(true)
                return remoteDevice
            }
            setDeviceRegistered(false)
        }

        val registeredDevice = gemDeviceApiClient.registerDevice(device)
        setDeviceRegistered(registeredDevice != null)
        return registeredDevice ?: device
    }

    private suspend fun setDeviceRegistered(isRegistered: Boolean = true) {
        context.dataStore.edit { it[Key.DeviceRegistered] = isRegistered }
    }

    private suspend fun reconcileSubscriptions(wallets: List<Wallet>) {
        val remoteSubscriptions = gemDeviceApiClient.getSubscriptions() ?: emptyList()
        val (toAdd, toRemove) = wallets.subscriptionsDiff(remoteSubscriptions)

        if (toAdd.isNotEmpty()) {
            gemDeviceApiClient.addSubscriptions(toAdd)
        }

        if (toRemove.isNotEmpty()) {
            gemDeviceApiClient.deleteSubscriptions(toRemove)
        }
    }

    private suspend fun resolvePushState(): PushState? {
        val pushEnabled = getPushEnabled().firstOrNull() ?: false
        val pushToken = if (pushEnabled) getPushToken() else ""

        if (pushEnabled && pushToken.isEmpty()) {
            requestPushToken.requestToken { token ->
                setPushToken(token)
                scope.launch { syncDevice() }
            }
            return null
        }

        return PushState(
            enabled = pushEnabled,
            token = pushToken,
        )
    }

    private fun getSubscriptionVersion(): Int {
        return configStore.getInt(ConfigKey.SubscriptionVersion.string)
    }

    private fun setSubscriptionVersion(subVersion: Int) {
        configStore.putInt(
            ConfigKey.SubscriptionVersion.string,
            subVersion
        )
    }

    private fun pushedDevice(): Device? {
        val raw = configStore.getString(ConfigKey.PushedDevice.string)
        if (raw.isEmpty()) {
            return null
        }
        return runCatching { jsonEncoder.decodeFromString(Device.serializer(), raw) }.getOrNull()
    }

    private fun pushedSubscriptionSignature(): String {
        return configStore.getString(ConfigKey.PushedSubscriptions.string)
    }

    private fun recordPushedState(device: Device, subscriptionSignature: String) {
        configStore.putString(ConfigKey.PushedDevice.string, jsonEncoder.encodeToString(Device.serializer(), device))
        configStore.putString(ConfigKey.PushedSubscriptions.string, subscriptionSignature)
    }

    private suspend fun buildDevice(
        pushToken: String,
        pushEnabled: Boolean,
        subscriptionsVersion: Int,
    ): Device {
        return Device(
            id = getDeviceId.getDeviceId(),
            platform = Platform.Android,
            platformStore = platformStore,
            os = Platform.os,
            model = Platform.model,
            token = pushToken,
            locale = getDeviceLocale(Locale.getDefault()),
            isPushEnabled = pushEnabled,
            isPriceAlertsEnabled = priceAlertRepository.isPriceAlertsEnabled().firstOrNull(),
            version = versionName,
            currency = getCurrentCurrency.getCurrentCurrency(),
            subscriptionsVersion = subscriptionsVersion,
        )
    }

    private fun Device.hasChanges(other: Device): Boolean = deviceHasChanges(this, other)

    internal enum class ConfigKey(val string: String) {
        PushToken("push_token"),
        PushedDevice("pushed_device"),
        PushedSubscriptions("pushed_subscriptions"),
        SubscriptionVersion("subscription_version"),
        ;
    }

    private object Key {
        val PushEnabled = booleanPreferencesKey("push_enabled")
        val DeviceRegistered = booleanPreferencesKey("device_registered")
    }

    companion object {
        fun getDeviceLocale(locale: Locale): DeviceLocale {
            val canonicalLocale = ULocale.addLikelySubtags(ULocale.forLocale(locale))
            val identifier = when (canonicalLocale.language) {
                "pt" -> "pt-BR"
                "zh" -> "${canonicalLocale.language}-${canonicalLocale.script}"
                else -> canonicalLocale.language
            }
            return DeviceLocale.entries.firstOrNull { it.string == identifier } ?: DeviceLocale.EN
        }
    }
}

private data class PushState(
    val enabled: Boolean,
    val token: String,
)

internal fun deviceHasChanges(current: Device, other: Device): Boolean {
    return current.id != other.id
            || current.token != other.token
            || current.locale != other.locale
            || current.version != other.version
            || current.currency != other.currency
            || current.isPushEnabled != other.isPushEnabled
            || current.isPriceAlertsEnabled != other.isPriceAlertsEnabled
            || current.subscriptionsVersion != other.subscriptionsVersion
}

internal fun List<Wallet>.subscriptionSignature(): String {
    return flatMap { wallet ->
        wallet.accounts.map { account -> "${wallet.id.id}/${account.chain.string}/${account.address}" }
    }
        .sorted()
        .joinToString(";")
}

// TODO: Temp solution. Move to App Layer with subscriptions subsystem when will prepared.
fun List<Wallet>.subscriptionsDiff(remote: List<WalletSubscriptionChains>): Pair<List<WalletSubscription>, List<WalletSubscriptionChains>> {
    val wallets = this

    val remoteIndex = remote.groupBy { it.walletId }
        .mapValues { item -> item.value.map { it.chains }.flatten() }

    val diffs = wallets.map { wallet -> walletSubscriptionsDiff(wallet, remoteIndex[wallet.id.id] ?: emptyList()) }
    val toRemove = diffs.map { it.second }.filter { it.chains.isNotEmpty() } +
            remote.filter { remote -> wallets.firstOrNull { it.id.id == remote.walletId } == null }

    val toAdd = diffs.map { it.first }.filter { it.subscriptions.isNotEmpty() }
    return Pair(toAdd, toRemove)
}

private fun walletSubscriptionsDiff(wallet: Wallet, remote: List<Chain>): Pair<WalletSubscription, WalletSubscriptionChains> {
    val toAdd = wallet.accounts.filter { !remote.contains(it.chain) }
        .groupBy { account ->  account.address }
        .map { entry ->
            AddressChains(entry.key, entry.value.map { it.chain })
        }

    val toRemove = remote.filter { wallet.getAccount(it) == null }
    return Pair(
        WalletSubscription(
            walletId = wallet.id.id,
            source = wallet.source,
            subscriptions = toAdd
        ),
        WalletSubscriptionChains(
            walletId = wallet.id.id,
            chains = toRemove
        ),
    )
}
