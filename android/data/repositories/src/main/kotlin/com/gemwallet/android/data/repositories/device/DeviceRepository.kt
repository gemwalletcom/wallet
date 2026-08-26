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
import com.gemwallet.android.data.service.store.ConfigStore
import com.gemwallet.android.ext.model
import com.gemwallet.android.ext.os
import com.gemwallet.android.model.NotificationsAvailable
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Device
import com.wallet.core.primitives.DeviceLocale
import com.wallet.core.primitives.Platform
import com.wallet.core.primitives.PlatformStore
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.launch
import uniffi.gemstone.GemDeviceService
import java.util.Locale

class DeviceRepository(
    private val context: Context,
    private val deviceService: GemDeviceService,
    private val deviceStore: GemstoneDeviceStore,
    private val configStore: ConfigStore,
    private val requestPushToken: RequestPushToken,
    private val platformStore: PlatformStore,
    private val notificationsAvailable: NotificationsAvailable,
    private val versionName: String,
    private val getDeviceId: GetDeviceId,
    private val priceAlertRepository: PriceAlertRepository,
    private val getCurrentCurrency: GetCurrentCurrency,
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
        val device = localDevice() ?: return
        if (!deviceService.needsSync(device.toJson())) {
            return
        }
        syncCoordinator.synchronize {
            val current = localDevice() ?: return@synchronize
            if (deviceService.needsSync(current.toJson())) {
                deviceService.sync(current.toJson())
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

    override suspend fun isDeviceRegistered(): Boolean = deviceStore.isRegistered()

    private suspend fun localDevice(): Device? {
        val pushState = resolvePushState() ?: return null
        return buildDevice(pushToken = pushState.token, pushEnabled = pushState.enabled)
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

    private suspend fun buildDevice(pushToken: String, pushEnabled: Boolean): Device {
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
            subscriptionsVersion = 0,
        )
    }

    internal enum class ConfigKey(val string: String) {
        PushToken("push_token"),
        ;
    }

    private object Key {
        val PushEnabled = booleanPreferencesKey("push_enabled")
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
