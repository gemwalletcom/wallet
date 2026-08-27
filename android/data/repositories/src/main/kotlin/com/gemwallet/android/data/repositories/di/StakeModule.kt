package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.data.repositories.gemstone.GemstoneStakeStore
import com.gemwallet.android.cases.stake.SyncStakeDelegations
import com.gemwallet.android.data.repositories.stake.StakeRepository
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.StakeDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemGateway
import uniffi.gemstone.GemAddressStore
import uniffi.gemstone.GemStakeService
import uniffi.gemstone.GemStakeStore
import uniffi.gemstone.GemStaticApiClient
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object StakeModule {
    @Singleton
    @Provides
    fun provideGemStakeStore(stakeDao: StakeDao, assetsDao: AssetsDao): GemStakeStore =
        GemstoneStakeStore(stakeDao, assetsDao)

    @Singleton
    @Provides
    fun provideGemStakeService(gateway: GemGateway, staticApiClient: GemStaticApiClient, store: GemStakeStore, addressStore: GemAddressStore): GemStakeService =
        GemStakeService(gateway, staticApiClient, store, addressStore)

    @Singleton
    @Provides
    fun provideStakeRepository(
        stakeDao: StakeDao,
        stakeService: GemStakeService,
    ): StakeRepository = StakeRepository(
        stakeDao = stakeDao,
        stakeService = stakeService,
    )

    @Singleton
    @Provides
    fun provideSyncStakeDelegations(stakeRepository: StakeRepository): SyncStakeDelegations = stakeRepository
}
