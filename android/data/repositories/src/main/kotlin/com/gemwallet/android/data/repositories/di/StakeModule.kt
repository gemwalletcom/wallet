package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.cases.addresses.SaveAddressNames
import com.gemwallet.android.data.repositories.gemstone.GemstoneStakeStore
import com.gemwallet.android.cases.stake.SyncStakeDelegations
import com.gemwallet.android.data.repositories.stake.StakeRepository
import com.gemwallet.android.data.service.store.database.StakeDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemGateway
import uniffi.gemstone.GemStakeService
import uniffi.gemstone.GemStakeStore
import uniffi.gemstone.GemStaticApiClient
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object StakeModule {
    @Singleton
    @Provides
    fun provideGemStakeStore(stakeDao: StakeDao, saveAddressNames: SaveAddressNames): GemStakeStore = GemstoneStakeStore(stakeDao, saveAddressNames)

    @Singleton
    @Provides
    fun provideGemStakeService(gateway: GemGateway, staticApiClient: GemStaticApiClient, store: GemStakeStore): GemStakeService =
        GemStakeService(gateway, staticApiClient, store)

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
