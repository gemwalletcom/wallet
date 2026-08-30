package com.gemwallet.android.data.services.gemstone.di

import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemPushNotificationService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object PushNotificationsModule {

    @Provides
    @Singleton
    fun provideGemPushNotificationService(): GemPushNotificationService = GemPushNotificationService()
}
