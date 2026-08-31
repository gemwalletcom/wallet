package com.gemwallet.android.data.services.gemstone.di

import com.gemwallet.android.data.service.store.database.AssetListDao
import com.gemwallet.android.data.service.store.database.SearchDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemSearchService
import uniffi.gemstone.GemSearchStore
import com.gemwallet.android.data.services.gemstone.stores.GemstoneSearchStore
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object TokensModule {
    @Provides
    @Singleton
    fun provideGemSearchStore(searchDao: SearchDao, assetListDao: AssetListDao): GemSearchStore = GemstoneSearchStore(searchDao, assetListDao)

}
