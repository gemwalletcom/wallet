package com.gemwallet.android

import android.app.Application
import androidx.lifecycle.DefaultLifecycleObserver
import androidx.lifecycle.LifecycleOwner
import androidx.lifecycle.ProcessLifecycleOwner
import coil3.ImageLoader
import coil3.PlatformContext
import coil3.SingletonImageLoader
import coil3.disk.DiskCache
import coil3.disk.directory
import coil3.memory.MemoryCache
import coil3.svg.SvgDecoder
import com.gemwallet.android.application.perpetual.coordinators.SyncPerpetualPositions
import com.gemwallet.android.application.transactions.coordinators.GetTransactions
import com.gemwallet.android.data.repositories.assets.TransactionPostProcessingService

import com.gemwallet.android.data.repositories.perpetual.HyperliquidObserverService
import com.gemwallet.android.data.repositories.stream.StreamObserverService
import dagger.hilt.android.HiltAndroidApp
import javax.inject.Inject

@HiltAndroidApp
class App : Application(), SingletonImageLoader.Factory {

    @Inject
    lateinit var streamObserver: StreamObserverService
    @Inject
    lateinit var hyperliquidObserver: HyperliquidObserverService
    @Inject
    lateinit var syncPerpetualPositions: SyncPerpetualPositions
    @Inject
    lateinit var getTransactions: GetTransactions
    @Inject
    lateinit var transactionPostProcessingService: TransactionPostProcessingService

    override fun onCreate() {
        super.onCreate()
        ProcessLifecycleOwner.get().lifecycle.addObserver(
            object : DefaultLifecycleObserver {
                override fun onStart(owner: LifecycleOwner) {
                    streamObserver.start()
                    hyperliquidObserver.start()
                }

                override fun onStop(owner: LifecycleOwner) {
                    streamObserver.stop()
                    hyperliquidObserver.stop()
                }
            },
        )
    }

    override fun newImageLoader(context: PlatformContext): ImageLoader {
        return ImageLoader.Builder(this)
            .components {
                add(SvgDecoder.Factory())
            }
            .memoryCache {
                MemoryCache.Builder()
                    .maxSizePercent(this, 0.25)
                    .build()
            }
            .diskCache {
                DiskCache.Builder()
                    .directory(cacheDir.resolve("image_cache"))
                    .maxSizeBytes(512L * 1024 * 1024) // 512Mb
                    .build()
            }
            .build()
    }

    companion object {
        init {
            System.loadLibrary("gemstone")
        }
    }
}
