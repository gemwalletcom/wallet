package com.gemwallet.android.flavors

import com.gemwallet.android.cases.device.GetPushEnabled
import com.gemwallet.android.cases.device.SetPushToken
import com.gemwallet.android.cases.device.SyncDeviceInfo
import com.gemwallet.android.cases.pushes.ShowSystemNotification
import com.gemwallet.android.model.PushNotificationField
import com.google.firebase.messaging.FirebaseMessagingService
import com.google.firebase.messaging.RemoteMessage
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import javax.inject.Inject

@AndroidEntryPoint
class FCM : FirebaseMessagingService() {

    @Inject
    lateinit var syncDeviceInfo: SyncDeviceInfo
    @Inject
    lateinit var getPushEnabled: GetPushEnabled
    @Inject
    lateinit var setPushToken: SetPushToken
    @Inject
    lateinit var showSystemNotification: ShowSystemNotification

    private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO)

    override fun onMessageReceived(message: RemoteMessage) {
        val pushEnabled = runBlocking { getPushEnabled.getPushEnabled().firstOrNull() == true }
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

    override fun onNewToken(token: String) {
        scope.launch {
            setPushToken.setPushToken(token)
            syncDeviceInfo.syncDeviceInfo()
        }
    }
}
