package com.gemwallet.android

import android.app.Application
import androidx.lifecycle.ProcessLifecycleOwner
import coil3.ImageLoader
import coil3.PlatformContext
import coil3.SingletonImageLoader
import coil3.disk.DiskCache
import coil3.disk.directory
import coil3.memory.MemoryCache
import coil3.network.okhttp.OkHttpNetworkFetcherFactory
import coil3.svg.SvgDecoder
import com.gemwallet.android.application.assets.cases.GetActiveAssetsInfo
import dagger.hilt.android.HiltAndroidApp
import okhttp3.Dispatcher
import okhttp3.OkHttpClient
import javax.inject.Inject

@HiltAndroidApp
class App : Application(), SingletonImageLoader.Factory {

    @Inject
    lateinit var appLifecycleCoordinator: AppLifecycleCoordinator
    @Inject
    lateinit var getActiveAssetsInfo: GetActiveAssetsInfo

    override fun onCreate() {
        super.onCreate()
        ProcessLifecycleOwner.get().lifecycle.addObserver(appLifecycleCoordinator)
    }

    override fun newImageLoader(context: PlatformContext): ImageLoader {
        return ImageLoader.Builder(this)
            .components {
                add(OkHttpNetworkFetcherFactory(callFactory = ::imageHttpClient))
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

    private fun imageHttpClient() = OkHttpClient.Builder()
        .dispatcher(Dispatcher().apply { maxRequestsPerHost = IMAGE_REQUESTS_PER_HOST })
        .build()

    companion object {
        private const val IMAGE_REQUESTS_PER_HOST = 16

        init {
            System.loadLibrary("gemstone")
        }
    }
}
