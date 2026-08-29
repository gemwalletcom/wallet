package com.gemwallet.android.data.repositories.di

import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemStakeRulesService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object StakeRulesModule {

    @Provides
    @Singleton
    fun provideGemStakeRulesService(): GemStakeRulesService = GemStakeRulesService()
}
