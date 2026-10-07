package com.gemwallet.android.flavors

import android.util.Log
import com.gemwallet.android.application.device.cases.GetPushEnabled
import com.gemwallet.android.application.device.cases.SetPushToken
import com.gemwallet.android.application.notifications.cases.ShowSystemNotification
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.model.PushNotificationField
import com.google.firebase.messaging.FirebaseMessagingService
import com.google.firebase.messaging.RemoteMessage
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import javax.inject.Inject

@AndroidEntryPoint
class FCM : FirebaseMessagingService() {
    @Inject
    lateinit var getPushEnabled: GetPushEnabled

    @Inject
    lateinit var setPushToken: SetPushToken

    @Inject
    lateinit var showSystemNotification: ShowSystemNotification

    private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.IO)

    override fun onMessageReceived(message: RemoteMessage) {
        val pushEnabled = runBlocking {
            runCatchingCancellable { getPushEnabled.getPushEnabled().firstOrNull() == true }
                .onFailure { Log.e("FCM", "Read push setting failed", it) }
                .getOrDefault(false)
        }
        if (!pushEnabled) {
            return
        }
        scope.launch {
            val type = message.data[PushNotificationField.Type.key]
            val rawData = message.data[PushNotificationField.Data.key]
            val title = message.notification?.title
            val subtitle = message.notification?.body
            showSystemNotification.showNotification(title, subtitle, type, rawData)
        }
    }

    @Deprecated("Deprecated in Java")
    override fun onNewToken(token: String) {
        setPushToken.setPushToken(token)
    }
}
