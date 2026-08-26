package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.data.repositories.notifications.InAppNotificationsRepository
import com.gemwallet.android.data.service.store.WalletPreferencesFactory
import com.gemwallet.android.data.service.store.database.InAppNotificationsDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemNotificationService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object InAppNotificationsModule {

    @Provides
    @Singleton
    fun provideInAppNotificationsRepository(
        notificationService: GemNotificationService,
        notificationsDao: InAppNotificationsDao,
        walletPreferencesFactory: WalletPreferencesFactory,
    ): InAppNotificationsRepository = InAppNotificationsRepository(
        notificationService = notificationService,
        notificationsDao = notificationsDao,
        walletPreferencesFactory = walletPreferencesFactory,
    )
}
