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
import com.gemwallet.android.data.service.store.ConfigStore
import com.gemwallet.android.ext.model
import com.gemwallet.android.ext.os
import com.gemwallet.android.model.NotificationsAvailable
import com.gemwallet.android.serializer.toJson
import dagger.Lazy
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
import uniffi.gemstone.GemDeviceInfo
import uniffi.gemstone.GemDevicePlatform
import uniffi.gemstone.GemDeviceService
import java.util.Locale

class DeviceRepository(
    private val context: Context,
    private val deviceService: Lazy<GemDeviceService>,
    private val configStore: ConfigStore,
    private val requestPushToken: RequestPushToken,
    private val platformStore: PlatformStore,
    private val notificationsAvailable: NotificationsAvailable,
    private val versionName: String,
    private val getDeviceId: GetDeviceId,
    private val getCurrentCurrency: GetCurrentCurrency,
    private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.IO),
) : SwitchPushEnabled,
    GetPushEnabled,
    GetPushToken,
    SetPushToken,
    IsDeviceRegistered,
    GemDevicePlatform
{
    private val Context.dataStore by preferencesDataStore(name = "device_config")

    override suspend fun switchPushEnabled(enabled: Boolean) {
        context.dataStore.edit { preferences ->
            preferences[Key.PushEnabled] = enabled && notificationsAvailable
        }
        try {
            deviceService.get().synchronizeIfNeeded()
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

    override suspend fun isDeviceRegistered(): Boolean = deviceService.get().isRegistered()

    override suspend fun deviceId(): String = getDeviceId.getDeviceId()

    override suspend fun deviceInfo(): GemDeviceInfo = GemDeviceInfo(
        platform = Platform.Android.toJson(),
        platformStore = platformStore.toJson(),
        os = Platform.os,
        model = Platform.model,
        version = versionName,
        locale = getDeviceLocale(Locale.getDefault()).toJson(),
    )

    override suspend fun pushToken(): String {
        val enabled = getPushEnabled().firstOrNull() ?: false
        val token = if (enabled) getPushToken() else ""
        if (enabled && token.isEmpty()) {
            requestPushToken.requestToken { requested ->
                setPushToken(requested)
                scope.launch { runCatching { deviceService.get().synchronizeIfNeeded() } }
            }
        }
        return token
    }

    override suspend fun isPushEnabled(): Boolean = getPushEnabled().firstOrNull() ?: false

    override suspend fun currency(): String = getCurrentCurrency.getCurrentCurrency().toJson()

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
