package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.config.coordinators.GetRemoteConfig
import com.gemwallet.android.data.coordinators.config.GetRemoteConfigImpl
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemConfigService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object ConfigModule {
    @Provides
    @Singleton
    fun provideGetRemoteConfig(
        configService: GemConfigService,
    ): GetRemoteConfig {
        return GetRemoteConfigImpl(
            configService = configService,
        )
    }
}
