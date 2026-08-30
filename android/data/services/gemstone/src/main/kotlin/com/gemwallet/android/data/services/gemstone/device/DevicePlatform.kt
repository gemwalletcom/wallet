package com.gemwallet.android.data.services.gemstone.device

import android.content.Context
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import com.gemwallet.android.application.device.cases.GetPushEnabled
import com.gemwallet.android.application.device.cases.GetPushToken
import com.gemwallet.android.application.device.cases.IsDeviceRegistered
import com.gemwallet.android.application.device.cases.RequestPushToken
import com.gemwallet.android.application.device.cases.SetPushToken
import com.gemwallet.android.application.device.cases.SwitchPushEnabled
import com.gemwallet.android.data.service.store.ConfigStore
import com.gemwallet.android.ext.model
import com.gemwallet.android.ext.os
import com.gemwallet.android.model.NotificationsAvailable
import com.gemwallet.android.serializer.toJson
import dagger.Lazy
import com.wallet.core.primitives.Platform
import com.wallet.core.primitives.PlatformStore
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.launch
import uniffi.gemstone.GemDeviceInfo
import uniffi.gemstone.GemDevicePlatform
import uniffi.gemstone.GemDeviceKeyService
import uniffi.gemstone.GemDeviceService
import java.util.Locale
import uniffi.gemstone.GemPreferencesService

class GemstoneDevicePlatform(
    private val context: Context,
    private val deviceService: Lazy<GemDeviceService>,
    private val configStore: ConfigStore,
    private val requestPushToken: RequestPushToken,
    private val platformStore: PlatformStore,
    private val notificationsAvailable: NotificationsAvailable,
    private val versionName: String,
    private val deviceKeyService: GemDeviceKeyService,
    private val preferencesService: GemPreferencesService,
    private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.IO),
) : SwitchPushEnabled,
    GetPushEnabled,
    GetPushToken,
    SetPushToken,
    IsDeviceRegistered,
    GemDevicePlatform
{
    private val Context.dataStore by preferencesDataStore(name = "device_config")

    private val pushEnabledState = MutableStateFlow(notificationsAvailable && preferencesService.isPushNotificationsEnabled())

    override suspend fun switchPushEnabled(enabled: Boolean) {
        setPushEnabled(enabled && notificationsAvailable)
        try {
            deviceService.get().synchronizeIfNeeded()
        } catch (_: Throwable) {}
    }

    override fun getPushEnabled(): Flow<Boolean> = pushEnabledState.onStart { migratePushEnabled() }

    private fun setPushEnabled(enabled: Boolean) {
        preferencesService.setPushNotificationsEnabled(enabled)
        pushEnabledState.value = enabled
    }

    private suspend fun migratePushEnabled() {
        val stored = context.dataStore.data.map { it[Key.PushEnabled] }.firstOrNull() ?: return
        if (stored && !preferencesService.isPushNotificationsEnabled()) {
            setPushEnabled(notificationsAvailable)
        }
        context.dataStore.edit { it.remove(Key.PushEnabled) }
    }

    override fun setPushToken(token: String) {
        configStore.putString(ConfigKey.PushToken.string, if (notificationsAvailable) token else "")
    }

    override suspend fun getPushToken(): String = configStore.getString(ConfigKey.PushToken.string)

    override suspend fun isDeviceRegistered(): Boolean = deviceService.get().isRegistered()

    override suspend fun deviceId(): String = deviceKeyService.deviceId()

    override suspend fun deviceInfo(): GemDeviceInfo = GemDeviceInfo(
        platform = Platform.Android.toJson(),
        platformStore = platformStore.toJson(),
        os = Platform.os,
        model = Platform.model,
        version = versionName,
        localeIdentifier = Locale.getDefault().toLanguageTag(),
    )

    override suspend fun pushToken(): String {
        val token = getPushToken()
        if (token.isEmpty()) {
            requestPushToken.requestToken { requested ->
                if (requested.isNotEmpty()) {
                    setPushToken(requested)
                    scope.launch { runCatching { deviceService.get().synchronizeIfNeeded() } }
                }
            }
        }
        return token
    }

    override suspend fun isPushEnabled(): Boolean = getPushEnabled().firstOrNull() ?: false

    override suspend fun currency(): String = preferencesService.getCurrency()

    internal enum class ConfigKey(val string: String) {
        PushToken("push_token"),
        ;
    }

    private object Key {
        val PushEnabled = booleanPreferencesKey("push_enabled")
    }
}
