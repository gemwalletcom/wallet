package com.gemwallet.android.data.repositories.device

import android.content.Context
import android.net.http.HttpException
import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.preferencesDataStore
import com.gemwallet.android.application.device.coordinators.GetDeviceId
import com.gemwallet.android.application.session.coordinators.GetCurrentCurrency
import com.gemwallet.android.cases.device.GetDeviceIdOld
import com.gemwallet.android.cases.device.GetPushEnabled
import com.gemwallet.android.cases.device.GetPushToken
import com.gemwallet.android.cases.device.RequestPushToken
import com.gemwallet.android.cases.device.SetPushToken
import com.gemwallet.android.cases.device.SwitchPushEnabled
import com.gemwallet.android.cases.device.SyncDeviceInfo
import com.gemwallet.android.cases.device.SyncSubscription
import com.gemwallet.android.data.repositories.config.UserConfig.Keys
import com.gemwallet.android.data.repositories.pricealerts.PriceAlertRepository
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.data.service.store.ConfigStore
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.model
import com.gemwallet.android.ext.os
import com.wallet.core.primitives.AddressChains
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Device
import com.wallet.core.primitives.MigrateDeviceIdRequest
import com.wallet.core.primitives.Platform
import com.wallet.core.primitives.PlatformStore
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletSubscription
import com.wallet.core.primitives.WalletSubscriptionChains
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.launch
import java.util.Locale
import kotlin.collections.firstOrNull
import kotlin.collections.map
import kotlin.collections.plus

class DeviceRepository(
    private val context: Context,
    private val gemDeviceApiClient: GemDeviceApiClient,
    private val configStore: ConfigStore,
    private val requestPushToken: RequestPushToken,
    private val platformStore: PlatformStore,
    private val versionName: String,
    private val getDeviceIdOld: GetDeviceIdOld,
    private val getDeviceId: GetDeviceId,
    private val priceAlertRepository: PriceAlertRepository,
    private val getCurrentCurrency: GetCurrentCurrency,
    private val walletsRepository: WalletsRepository,
) : SyncDeviceInfo,
    SwitchPushEnabled,
    GetPushEnabled,
    GetPushToken,
    SetPushToken,
    SyncSubscription
{
    private val Context.dataStore by preferencesDataStore(name = "device_config")

    private suspend fun migrate() {
        try {
            if (getDeviceIdOld.isMigrated()) return

            if (getDeviceIdOld.getDeviceId().isEmpty()) return

            gemDeviceApiClient.migrateDevice(
                MigrateDeviceIdRequest(
                    oldDeviceId = getDeviceIdOld.getDeviceId(),
                    publicKey = getDeviceId.getDeviceId()
                )
            )
            getDeviceIdOld.migrated()
        } catch (_: Throwable) { }
    }

    override suspend fun syncDeviceInfo() {
        synchronizeDevice(
            wallets = loadWallets(),
            shouldInvalidateSubscriptions = false,
        )
    }

    override suspend fun switchPushEnabled(enabled: Boolean, wallets: List<Wallet>) {
        context.dataStore.edit { preferences ->
            preferences[Key.PushEnabled] = enabled
        }
        try {
            syncDeviceInfo()
        } catch (_: Throwable) {}
    }

    override fun getPushEnabled(): Flow<Boolean> = context.dataStore.data
        .map { preferences -> preferences[Key.PushEnabled] == true }

    override fun setPushToken(token: String) {
        configStore.putString(ConfigKey.PushToken.string, token)
    }

    override suspend fun getPushToken(): String {
        return if (getPushEnabled().firstOrNull() == true) {
            configStore.getString(ConfigKey.PushToken.string)
        } else {
            ""
        }
    }

    override suspend fun syncSubscription(wallets: List<Wallet>) {
        synchronizeDevice(
            wallets = walletsForSubscriptionSync(
                storedWallets = loadWallets(),
                requestedWallets = wallets,
            ),
            shouldInvalidateSubscriptions = true,
        )
    }

    private suspend fun synchronizeDevice(
        wallets: List<Wallet>,
        shouldInvalidateSubscriptions: Boolean,
        pushTokenOverride: String? = null,
    ) {
        try {
            migrate()

            if (shouldInvalidateSubscriptions) {
                invalidateSubscriptions()
            }

            val subscriptionState = getSubscriptionSyncState()
            val pushState = resolvePushState(
                wallets = wallets,
                pushTokenOverride = pushTokenOverride,
            ) ?: return
            val localDevice = buildLocalDevice(
                pushState = pushState,
                subscriptionsVersion = subscriptionState.version,
            )
            val remoteDevice = getOrCreateDevice(localDevice)
            val didSyncSubscriptions = maybeSyncSubscriptions(
                wallets = wallets,
                subscriptionState = subscriptionState,
                remoteDevice = remoteDevice,
            )
            val requestDevice = buildDeviceUpdateRequest(
                localDevice = localDevice,
                remoteDevice = remoteDevice,
                localSubscriptionVersion = subscriptionState.version,
                didSyncSubscriptions = didSyncSubscriptions,
            )

            updateDevice(
                remote = remoteDevice,
                request = requestDevice,
            )
        } catch (_: Throwable) {}
    }

    private suspend fun loadWallets(): List<Wallet> {
        return walletsRepository.getAll().firstOrNull() ?: emptyList()
    }

    @Throws(HttpException::class)
    private suspend fun isDeviceRegistered(): Boolean {
        val local = context.dataStore.data.map { it[Key.DeviceRegistered] }.firstOrNull() == true
        return local || gemDeviceApiClient.isDeviceRegistered()
    }

    private suspend fun getOrCreateDevice(device: Device): Device {
        if (isDeviceRegistered()) {
            gemDeviceApiClient.getDevice()?.let { remoteDevice ->
                setDeviceRegistered(true)
                return remoteDevice
            }
            setDeviceRegistered(false)
        }

        val registeredDevice = gemDeviceApiClient.registerDevice(device) ?: device
        setDeviceRegistered(gemDeviceApiClient.isDeviceRegistered())
        return registeredDevice
    }

    private suspend fun updateDevice(remote: Device, request: Device) {
        if (!remote.hasChanges(request)) {
            return
        }

        gemDeviceApiClient.updateDevice(request = request)
        setDeviceRegistered(true)
    }

    private suspend fun setDeviceRegistered(isRegistered: Boolean = true) {
        context.dataStore.edit { it[Key.DeviceRegistered] = isRegistered }
    }

    private suspend fun reconcileSubscriptions(wallets: List<Wallet>): Boolean {
        return try {
            val remoteSubscriptions = gemDeviceApiClient.getSubscriptions() ?: emptyList()
            val (toAdd, toRemove) = wallets.subscriptionsDiff(remoteSubscriptions)

            if (toAdd.isNotEmpty()) {
                gemDeviceApiClient.addSubscriptions(toAdd)
            }

            if (toRemove.isNotEmpty()) {
                gemDeviceApiClient.deleteSubscriptions(toRemove)
            }

            setSubscriptionVersionHasChange(false)
            true
        } catch (_: Throwable) {
            false
        }
    }

    private suspend fun resolvePushState(
        wallets: List<Wallet>,
        pushTokenOverride: String?,
    ): PushState? {
        val pushEnabled = getPushEnabled().firstOrNull() ?: false
        val pushToken = pushTokenOverride ?: getPushToken()

        if (pushEnabled && pushToken.isEmpty() && pushTokenOverride == null) {
            requestPushToken.requestToken { token ->
                setPushToken(token)
                CoroutineScope(Dispatchers.IO).launch {
                    synchronizeDevice(
                        wallets = wallets,
                        shouldInvalidateSubscriptions = false,
                        pushTokenOverride = token,
                    )
                }
            }
            return null
        }

        return PushState(
            enabled = pushEnabled,
            token = pushToken,
        )
    }

    private suspend fun buildLocalDevice(
        pushState: PushState,
        subscriptionsVersion: Int,
    ): Device {
        return buildDevice(
            pushToken = pushState.token,
            pushEnabled = pushState.enabled,
            subscriptionsVersion = subscriptionsVersion,
        )
    }

    private suspend fun maybeSyncSubscriptions(
        wallets: List<Wallet>,
        subscriptionState: SubscriptionSyncState,
        remoteDevice: Device,
    ): Boolean {
        if (!subscriptionState.shouldSync(remoteDevice.subscriptionsVersion)) {
            return false
        }

        return reconcileSubscriptions(wallets)
    }

    private fun buildDeviceUpdateRequest(
        localDevice: Device,
        remoteDevice: Device,
        localSubscriptionVersion: Int,
        didSyncSubscriptions: Boolean,
    ): Device {
        return localDevice.copy(
            subscriptionsVersion = subscriptionVersionForDeviceUpdate(
                localVersion = localSubscriptionVersion,
                remoteVersion = remoteDevice.subscriptionsVersion,
                useLocalVersion = didSyncSubscriptions,
            )
        )
    }

    private fun getSubscriptionSyncState(): SubscriptionSyncState {
        return SubscriptionSyncState(
            version = getSubscriptionVersion(),
            hasPendingChanges = hasPendingSubscriptionChanges(),
        )
    }

    private fun getSubscriptionVersion(): Int {
        return configStore.getInt(Keys.SubscriptionVersion.string)
    }

    private fun setSubscriptionVersion(subVersion: Int) {
        configStore.putInt(
            Keys.SubscriptionVersion.string,
            subVersion
        )
    }

    private fun hasPendingSubscriptionChanges(): Boolean {
        return configStore.getBoolean(Keys.SubscriptionVersionHasChange.string)
    }

    private fun setSubscriptionVersionHasChange(hasChange: Boolean) {
        configStore.putBoolean(Keys.SubscriptionVersionHasChange.string, hasChange)
    }

    private fun invalidateSubscriptions() {
        val invalidatedState = getSubscriptionSyncState().invalidate()
        setSubscriptionVersion(invalidatedState.version)
        setSubscriptionVersionHasChange(invalidatedState.hasPendingChanges)
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
            locale = getLocale(Locale.getDefault()),
            isPushEnabled = pushEnabled,
            isPriceAlertsEnabled = priceAlertRepository.isPriceAlertsEnabled().firstOrNull(),
            version = versionName,
            currency = getCurrentCurrency.getCurrentCurrency().string,
            subscriptionsVersion = subscriptionsVersion,
        )
    }

    private fun Device.hasChanges(other: Device): Boolean = deviceHasChanges(this, other)

    internal enum class ConfigKey(val string: String) {
        DeviceId("device-uuid"),
        PushToken("push_token"),
        ;
    }

    private object Key {
        val PushEnabled = booleanPreferencesKey("push_enabled")
        val DeviceRegistered = booleanPreferencesKey("device_registered")
    }

    companion object {
        fun getLocale(locale: Locale): String {
            val tag = locale.toLanguageTag()
            if (tag == "pt-BR" || tag == "pt_BR") {
                return "pt-BR"
            }
            if (locale.language == "zh") {
                return "${locale.language}-${(locale.script.ifEmpty { "Hans" })}"
            }
            return  locale.language
        }
    }
}

private data class PushState(
    val enabled: Boolean,
    val token: String,
)

internal data class SubscriptionSyncState(
    val version: Int,
    val hasPendingChanges: Boolean,
) {
    fun invalidate(): SubscriptionSyncState {
        return if (hasPendingChanges) {
            this
        } else {
            copy(
                version = version + 1,
                hasPendingChanges = true,
            )
        }
    }

    fun shouldSync(remoteVersion: Int): Boolean {
        return hasPendingChanges || remoteVersion != version
    }
}

internal fun subscriptionVersionForDeviceUpdate(
    localVersion: Int,
    remoteVersion: Int?,
    useLocalVersion: Boolean,
): Int {
    return if (useLocalVersion || remoteVersion == null) {
        localVersion
    } else {
        remoteVersion
    }
}

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

internal fun walletsForSubscriptionSync(
    storedWallets: List<Wallet>,
    requestedWallets: List<Wallet>,
): List<Wallet> {
    if (storedWallets.isEmpty()) return requestedWallets
    if (requestedWallets.isEmpty()) return storedWallets

    return (storedWallets + requestedWallets)
        .associateBy { it.id }
        .values
        .toList()
}

// TODO: Temp solution. Move to App Layer with subscriptions subsystem when will prepared.
fun List<Wallet>.subscriptionsDiff(remote: List<WalletSubscriptionChains>): Pair<List<WalletSubscription>, List<WalletSubscriptionChains>> {
    val wallets = this

    val remoteIndex = remote.groupBy { it.walletId }
        .mapValues { item -> item.value.map { it.chains }.flatten() }

    val diffs = wallets.map { wallet -> walletSubscriptionsDiff(wallet, remoteIndex[wallet.id] ?: emptyList()) }
    val toRemove = diffs.map { it.second }.filter { it.chains.isNotEmpty() } +
            remote.filter { remote -> wallets.firstOrNull { it.id == remote.walletId} == null }

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
            walletId = wallet.id,
            source = wallet.source,
            subscriptions = toAdd
        ),
        WalletSubscriptionChains(
            walletId = wallet.id,
            chains = toRemove
        ),
    )
}
