package com.gemwallet.android.data.services.gemstone.device

import com.gemwallet.android.application.device.cases.EnablePushForNewWallet
import com.gemwallet.android.application.device.cases.EnablePushForSupport
import com.gemwallet.android.application.device.cases.GetPushEnabled
import com.gemwallet.android.application.device.cases.SetPushToken
import com.gemwallet.android.application.device.cases.SwitchPushEnabled
import com.gemwallet.android.data.services.store.ConfigStore
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.model.NotificationsAvailable
import dagger.Lazy
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemDeviceService
import uniffi.gemstone.GemNotificationsService
import uniffi.gemstone.GemPushState

class DevicePushSettings(
    private val configStore: ConfigStore,
    private val notificationsAvailable: NotificationsAvailable,
    private val deviceService: Lazy<GemDeviceService>,
    private val notificationsService: Lazy<GemNotificationsService>,
    private val ioDispatcher: CoroutineDispatcher = Dispatchers.IO,
    private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + ioDispatcher),
) : SwitchPushEnabled,
    EnablePushForSupport,
    EnablePushForNewWallet,
    GetPushEnabled,
    SetPushToken {

    private val pushEnabledState = MutableStateFlow(false)

    override suspend fun enablePushForSupport(): GemPushState? = withContext(ioDispatcher) {
        notificationsService.get().askToEnable()?.also { state ->
            pushEnabledState.value = state.isEnabled
        }
    }

    override fun enablePushForNewWallet() {
        scope.launch {
            notificationsService.get().askToEnable()?.let { state -> pushEnabledState.value = state.isEnabled }
        }
    }

    override suspend fun switchPushEnabled(enabled: Boolean): GemPushState = withContext(ioDispatcher) {
        val state = notificationsService.get().setEnabled(enabled)
        pushEnabledState.value = state.isEnabled
        state
    }

    override fun getPushEnabled(): Flow<Boolean> = pushEnabledState.onStart {
        pushEnabledState.value = notificationsService.get().isEnabled()
    }.flowOn(ioDispatcher)

    override fun setPushToken(token: String) {
        val stored = if (notificationsAvailable) token else ""
        if (stored == configStore.getString(PUSH_TOKEN)) {
            return
        }
        configStore.putString(PUSH_TOKEN, stored)
        scope.launch { runCatchingCancellable { deviceService.get().synchronizeIfNeeded() } }
    }

    suspend fun getPushToken(): String = configStore.getString(PUSH_TOKEN)

    companion object {
        const val PREFERENCES_FILE_NAME = "device-info"
        private const val PUSH_TOKEN = "push_token"
    }
}
