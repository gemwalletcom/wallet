package com.gemwallet.android.data.repositories.di

import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemChainService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object ChainModule {

    @Provides
    @Singleton
    fun provideGemChainService(): GemChainService = GemChainService()
}
