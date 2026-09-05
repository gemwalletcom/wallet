package com.gemwallet.android.data.services.gemstone.device

import com.gemwallet.android.ext.toGem
import android.content.Context
import androidx.core.app.NotificationManagerCompat
import com.gemwallet.android.application.device.cases.GetPushToken
import com.gemwallet.android.application.device.cases.RequestPushToken
import com.gemwallet.android.application.device.cases.SetPushToken
import com.gemwallet.android.ext.model
import com.gemwallet.android.ext.os
import com.gemwallet.android.model.NotificationsAvailable
import com.wallet.core.primitives.Platform
import com.wallet.core.primitives.PlatformStore
import uniffi.gemstone.GemDeviceInfo
import uniffi.gemstone.GemDevicePlatform
import uniffi.gemstone.GemDeviceKeyService
import java.util.Locale
import uniffi.gemstone.GemPreferencesService

class GemstoneDevicePlatform(
    private val context: Context,
    private val getPushToken: GetPushToken,
    private val setPushToken: SetPushToken,
    private val requestPushToken: RequestPushToken,
    private val platformStore: PlatformStore,
    private val notificationsAvailable: NotificationsAvailable,
    private val versionName: String,
    private val deviceKeyService: GemDeviceKeyService,
    private val preferencesService: GemPreferencesService,
) : GemDevicePlatform {

    override suspend fun deviceId(): String = deviceKeyService.deviceId()

    override suspend fun deviceInfo(): GemDeviceInfo = GemDeviceInfo(
        platform = Platform.Android.toGem(),
        platformStore = platformStore.toGem(),
        os = Platform.os,
        model = Platform.model,
        version = versionName,
        localeIdentifier = Locale.getDefault().toLanguageTag(),
    )

    override suspend fun pushToken(): String {
        val token = getPushToken.getPushToken()
        if (token.isEmpty()) {
            requestPushToken.requestToken { requested ->
                if (requested.isNotEmpty()) {
                    setPushToken.setPushToken(requested)
                }
            }
        }
        return token
    }

    override suspend fun isPushEnabled(): Boolean =
        notificationsAvailable &&
            preferencesService.isPushNotificationsEnabled() &&
            NotificationManagerCompat.from(context).areNotificationsEnabled()

    override suspend fun getCurrency(): String = preferencesService.getCurrency()
}
