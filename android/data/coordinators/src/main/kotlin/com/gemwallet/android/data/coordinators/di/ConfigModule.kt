package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.config.coordinators.GetRemoteConfig
import com.gemwallet.android.data.coordinators.config.GetRemoteConfigImpl
import com.gemwallet.android.data.services.gemapi.GemApiClient
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object ConfigModule {
    @Provides
    @Singleton
    fun provideGetRemoteConfig(
        gemApiClient: GemApiClient,
    ): GetRemoteConfig {
        return GetRemoteConfigImpl(
            gemApiClient = gemApiClient,
        )
    }
}
