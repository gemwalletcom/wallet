package com.reown.android

import android.app.Application
import com.reown.android.relay.ConnectionType
import com.reown.walletkit.client.Wallet

object Core {
    object Model {
        data class AppMetaData(
            val name: String,
            val description: String,
            val url: String,
            val icons: List<String>,
            val redirect: String? = null,
        )
    }
}

object CoreClient {
    interface CoreDelegate {
        fun onConnectionStateChange(state: Wallet.Model.ConnectionState)
        fun onError(error: Wallet.Model.Error)
    }

    fun initialize(
        application: Application,
        projectId: String,
        metaData: Core.Model.AppMetaData,
        connectionType: ConnectionType,
        telemetryEnabled: Boolean,
        onSuccess: () -> Unit,
    ) {
        onSuccess()
    }

    fun setDelegate(delegate: CoreDelegate) = Unit
}
