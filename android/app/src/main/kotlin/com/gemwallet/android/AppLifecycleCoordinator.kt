package com.gemwallet.android

import android.content.Context
import androidx.glance.appwidget.updateAll
import androidx.lifecycle.DefaultLifecycleObserver
import androidx.lifecycle.LifecycleOwner
import com.gemwallet.android.data.repositories.perpetual.HyperliquidObserverService
import com.gemwallet.android.data.repositories.stream.StreamObserverService
import com.gemwallet.android.features.widgets.PricesWidget
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class AppLifecycleCoordinator @Inject constructor(
    @ApplicationContext private val context: Context,
    private val streamObserver: StreamObserverService,
    private val hyperliquidObserver: HyperliquidObserverService,
) : DefaultLifecycleObserver {

    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)

    override fun onStart(owner: LifecycleOwner) {
        streamObserver.start()
        hyperliquidObserver.start()
    }

    override fun onStop(owner: LifecycleOwner) {
        streamObserver.stop()
        hyperliquidObserver.stop()
        scope.launch { PricesWidget().updateAll(context) }
    }
}
