package com.gemwallet.android.data.services.gemstone.di

import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemApiClient
import uniffi.gemstone.GemWidgetService
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
object WidgetModule {

    @Provides
    @Singleton
    fun provideGemWidgetService(apiClient: GemApiClient): GemWidgetService = GemWidgetService(apiClient)
}
