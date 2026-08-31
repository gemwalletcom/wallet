package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.update.cases.ObserveAppUpdateOffer
import com.gemwallet.android.application.update.cases.SkipAppUpdate
import com.gemwallet.android.application.update.cases.SyncAppUpdate
import com.gemwallet.android.data.coordinators.update.AppUpdateCoordinator
import com.gemwallet.android.model.BuildInfo
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemAppUpdateService
import uniffi.gemstone.GemConfigService
import uniffi.gemstone.GemPreferencesService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object UpdateModule {

    @Provides
    @Singleton
    fun provideGemAppUpdateService(
        configService: GemConfigService,
        preferencesService: GemPreferencesService,
    ): GemAppUpdateService = GemAppUpdateService(configService, preferencesService)

    @Provides
    @Singleton
    fun provideAppUpdateCoordinator(
        appUpdateService: GemAppUpdateService,
        buildInfo: BuildInfo,
    ): AppUpdateCoordinator = AppUpdateCoordinator(appUpdateService, buildInfo)

    @Provides
    @Singleton
    fun provideSyncAppUpdate(coordinator: AppUpdateCoordinator): SyncAppUpdate = coordinator

    @Provides
    @Singleton
    fun provideObserveAppUpdateOffer(coordinator: AppUpdateCoordinator): ObserveAppUpdateOffer = coordinator

    @Provides
    @Singleton
    fun provideSkipAppUpdate(coordinator: AppUpdateCoordinator): SkipAppUpdate = coordinator
}
