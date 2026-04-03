package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.blockchain.services.StakeService
import com.gemwallet.android.data.repositories.stake.StakeRepository
import com.gemwallet.android.data.service.store.database.StakeDao
import com.gemwallet.android.data.services.gemapi.GemApiStaticClient
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemGateway
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object StakeModule {
    @Singleton
    @Provides
    fun provideStakeRepository(
        stakeDao: StakeDao,
        gateway: GemGateway,
        gemApiStaticClient: GemApiStaticClient,
    ): StakeRepository = StakeRepository(
        stakeDao = stakeDao,
        gemApiStaticClient = gemApiStaticClient,
        stakeService = StakeService(gateway),
    )
}
