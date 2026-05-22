package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.data.repositories.notifications.InAppNotificationsRepository
import com.gemwallet.android.data.service.store.WalletPreferencesFactory
import com.gemwallet.android.data.service.store.database.InAppNotificationsDao
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object InAppNotificationsModule {

    @Provides
    @Singleton
    fun provideInAppNotificationsRepository(
        gemDeviceApiClient: GemDeviceApiClient,
        notificationsDao: InAppNotificationsDao,
        walletPreferencesFactory: WalletPreferencesFactory,
    ): InAppNotificationsRepository = InAppNotificationsRepository(
        gemDeviceApiClient = gemDeviceApiClient,
        notificationsDao = notificationsDao,
        walletPreferencesFactory = walletPreferencesFactory,
    )
}
