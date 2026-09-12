package com.gemwallet.android.data.services.gemstone.device

import android.content.Context
import android.util.Log
import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.preferencesDataStore
import com.gemwallet.android.application.device.cases.GetPushEnabled
import com.gemwallet.android.application.device.cases.GetPushToken
import com.gemwallet.android.application.device.cases.SetPushToken
import com.gemwallet.android.application.device.cases.SwitchPushEnabled
import com.gemwallet.android.data.service.store.ConfigStore
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.model.NotificationsAvailable
import dagger.Lazy
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.launch
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onStart
import uniffi.gemstone.GemDeviceService
import uniffi.gemstone.GemNotificationsService
import uniffi.gemstone.GemPreferencesService

class DevicePushSettings(
    private val context: Context,
    private val configStore: ConfigStore,
    private val notificationsAvailable: NotificationsAvailable,
    private val preferencesService: GemPreferencesService,
    private val deviceService: Lazy<GemDeviceService>,
    private val notificationsService: Lazy<GemNotificationsService>,
    private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.IO),
) : SwitchPushEnabled, GetPushEnabled, GetPushToken, SetPushToken {

    private val Context.dataStore by preferencesDataStore(name = "device_config")

    private val pushEnabledState = MutableStateFlow(false)

    override suspend fun switchPushEnabled(enabled: Boolean) {
        pushEnabledState.value = runCatchingCancellable {
            notificationsService.get().setEnabled(enabled)
        }.getOrElse {
            Log.e(TAG, "push notifications toggle failed", it)
            notificationsService.get().isEnabled()
        }
    }

    override fun getPushEnabled(): Flow<Boolean> = pushEnabledState.onStart {
        migratePushEnabled()
        pushEnabledState.value = notificationsService.get().isEnabled()
    }

    override fun setPushToken(token: String) {
        val stored = if (notificationsAvailable) token else ""
        if (stored == configStore.getString(PUSH_TOKEN)) {
            return
        }
        configStore.putString(PUSH_TOKEN, stored)
        scope.launch { runCatching { deviceService.get().synchronizeIfNeeded() } }
    }

    override suspend fun getPushToken(): String = configStore.getString(PUSH_TOKEN)

    private suspend fun migratePushEnabled() {
        val stored = context.dataStore.data.map { it[LegacyPushEnabled] }.firstOrNull() ?: return
        if (stored && !preferencesService.isPushNotificationsEnabled()) {
            preferencesService.setPushNotificationsEnabled(true)
        }
        context.dataStore.edit { it.remove(LegacyPushEnabled) }
    }

    private companion object {
        const val TAG = "PushSettings"
        const val PUSH_TOKEN = "push_token"
        val LegacyPushEnabled = booleanPreferencesKey("push_enabled")
    }
}
