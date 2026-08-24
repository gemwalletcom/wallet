package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.config.coordinators.GetRemoteConfig
import com.gemwallet.android.application.update.coordinators.ObserveAppUpdateOffer
import com.gemwallet.android.application.update.coordinators.SkipAppUpdate
import com.gemwallet.android.application.update.coordinators.SyncAppUpdate
import com.gemwallet.android.data.coordinators.update.AppUpdateCoordinator
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.model.BuildInfo
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object UpdateModule {

    @Provides
    @Singleton
    fun provideAppUpdateCoordinator(
        getRemoteConfig: GetRemoteConfig,
        userConfig: UserConfig,
        buildInfo: BuildInfo,
    ): AppUpdateCoordinator {
        return AppUpdateCoordinator(
            getRemoteConfig = getRemoteConfig,
            userConfig = userConfig,
            buildInfo = buildInfo,
        )
    }

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
