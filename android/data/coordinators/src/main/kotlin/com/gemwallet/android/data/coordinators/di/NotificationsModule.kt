package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.notifications.cases.GetInAppNotifications
import com.gemwallet.android.data.coordinators.notifications.GetInAppNotificationsImpl
import com.gemwallet.android.data.repositories.gemstone.GemstoneNotificationStore
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object NotificationsModule {

    @Provides
    @Singleton
    fun provideGetInAppNotifications(notificationStore: GemstoneNotificationStore): GetInAppNotifications =
        GetInAppNotificationsImpl(notificationStore)
}
