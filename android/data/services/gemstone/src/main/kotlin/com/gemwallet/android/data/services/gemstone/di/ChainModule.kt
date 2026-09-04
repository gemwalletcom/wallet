package com.gemwallet.android.data.services.gemstone.di

import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemChainService
import uniffi.gemstone.GemChainServiceInterface
import uniffi.gemstone.GemDeeplinkService
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
    fun provideGemChainServiceInterface(service: GemChainService): GemChainServiceInterface = service
}
