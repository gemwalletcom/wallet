package com.gemwallet.android.data.services.gemapi.di

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
    ): OkHttpClient = OkHttpClient.Builder()
        .connectionPool(ConnectionPool(32, 5, TimeUnit.MINUTES))
        .cache(Cache(context.cacheDir, 10 * 1024 * 1024))
        .connectTimeout(60, TimeUnit.SECONDS)
        .readTimeout(120, TimeUnit.SECONDS)
        .writeTimeout(120, TimeUnit.SECONDS)
        .addInterceptor { chain ->
            chain.proceed(
                chain.request()
                    .newBuilder()
                    .header("User-Agent", "Gem/${buildInfo.versionCode} Android/${Build.VERSION.RELEASE} Version/${buildInfo.versionName}")
                    .build()
            )
        }
        .build()


}
