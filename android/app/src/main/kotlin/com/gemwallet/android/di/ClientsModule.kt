package com.gemwallet.android.di

import android.content.Context
import android.os.Build
import com.gemwallet.android.model.BuildInfo
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import okhttp3.Cache
import okhttp3.ConnectionPool
import okhttp3.OkHttpClient
import java.util.concurrent.TimeUnit
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object ClientsModule {

    @Provides
    @Singleton
    fun provideGemHttpClient(
        @ApplicationContext context: Context,
        buildInfo: BuildInfo,
    ): OkHttpClient {
        val userAgent = "Gem/${buildInfo.versionCode} Android/${Build.VERSION.RELEASE} Version/${buildInfo.versionName}"
        return OkHttpClient.Builder()
            .connectionPool(ConnectionPool(MAX_IDLE_CONNECTIONS, KEEP_ALIVE_MINUTES, TimeUnit.MINUTES))
            .cache(Cache(context.cacheDir, CACHE_SIZE_BYTES))
            .connectTimeout(CONNECT_TIMEOUT_SECONDS, TimeUnit.SECONDS)
            .readTimeout(READ_TIMEOUT_SECONDS, TimeUnit.SECONDS)
            .writeTimeout(WRITE_TIMEOUT_SECONDS, TimeUnit.SECONDS)
            .addInterceptor { chain ->
                chain.proceed(
                    chain.request()
                        .newBuilder()
                        .header("User-Agent", userAgent)
                        .build()
                )
            }
            .build()
    }

    private const val MAX_IDLE_CONNECTIONS = 32
    private const val KEEP_ALIVE_MINUTES = 5L
    private const val CACHE_SIZE_BYTES = 10L * 1024 * 1024
    private const val CONNECT_TIMEOUT_SECONDS = 60L
    private const val READ_TIMEOUT_SECONDS = 120L
    private const val WRITE_TIMEOUT_SECONDS = 120L
}
