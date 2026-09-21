package com.gemwallet.android.data.services.gemstone.di

import com.gemwallet.android.data.service.store.database.NodesDao
import com.gemwallet.android.data.services.gemstone.stores.GemstonePreferencesStore
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.Config
import uniffi.gemstone.GemNodeService
import uniffi.gemstone.GemNodeServiceInterface
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object NodesModule {

    @Provides
    @Singleton
    fun provideGemstoneConfig(): Config = Config()

    @Provides
    @Singleton
    fun provideNodeService(nodesDao: NodesDao, preferences: GemstonePreferencesStore): GemNodeService = GemNodeService(nodesDao, preferences)

    @Provides
    fun provideGemNodeServiceInterface(service: GemNodeService): GemNodeServiceInterface = service
}
