package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.blockchain.services.StakeService
import com.gemwallet.android.cases.stake.SyncStakeDelegations
import com.gemwallet.android.data.repositories.stake.StakeRepository
import com.gemwallet.android.data.service.store.database.StakeDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemGateway
import uniffi.gemstone.GemStaticAssetsService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object StakeModule {
    @Singleton
    @Provides
    fun provideStakeRepository(
        stakeDao: StakeDao,
        gateway: GemGateway,
        staticAssetsService: GemStaticAssetsService,
    ): StakeRepository = StakeRepository(
        stakeDao = stakeDao,
        staticAssetsService = staticAssetsService,
        stakeService = StakeService(gateway),
    )

    @Singleton
    @Provides
    fun provideSyncStakeDelegations(stakeRepository: StakeRepository): SyncStakeDelegations = stakeRepository
}
