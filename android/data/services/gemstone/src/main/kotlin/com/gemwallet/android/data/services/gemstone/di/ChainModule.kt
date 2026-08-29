package com.gemwallet.android.data.services.gemstone.di

import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemChainService
import uniffi.gemstone.GemDeeplinkService
import uniffi.gemstone.GemSwapSelectionService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object ChainModule {

    @Provides
    @Singleton
    fun provideGemChainService(): GemChainService = GemChainService()

    @Provides
    @Singleton
    fun provideGemDeeplinkService(): GemDeeplinkService = GemDeeplinkService()

    @Provides
    @Singleton
    fun provideGemSwapSelectionService(): GemSwapSelectionService = GemSwapSelectionService()
}
