package com.gemwallet.android

import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import com.gemwallet.android.model.PushNotificationField
import dagger.hilt.android.AndroidEntryPoint
import javax.inject.Inject

@AndroidEntryPoint
class NotificationTapActivity : ComponentActivity() {

    @Inject lateinit var pendingNavigationCoordinator: PendingNavigationCoordinator

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        intent.getStringExtra(PushNotificationField.Type.key)?.let { type ->
            pendingNavigationCoordinator.pendNotification(type, intent.getStringExtra(PushNotificationField.Data.key))
        }
        startActivity(Intent(this, MainActivity::class.java).setFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_SINGLE_TOP))
        finish()
    }
}

internal fun Intent.putNotificationPayload(type: String?, rawData: String?): Intent = apply {
    type?.let { putExtra(PushNotificationField.Type.key, it) }
    rawData?.let { putExtra(PushNotificationField.Data.key, it) }
}
