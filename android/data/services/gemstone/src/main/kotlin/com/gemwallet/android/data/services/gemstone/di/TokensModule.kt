package com.gemwallet.android.data.services.gemstone.di

import com.gemwallet.android.data.services.store.database.AssetListDao
import com.gemwallet.android.data.services.store.database.SearchDao
import com.gemwallet.android.data.services.gemstone.stores.GemstoneSearchStore
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemSearchService
import uniffi.gemstone.GemSearchStore
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object TokensModule {
    @Provides
    @Singleton
    fun provideGemSearchStore(searchDao: SearchDao, assetListDao: AssetListDao): GemSearchStore = GemstoneSearchStore(searchDao, assetListDao)
}
