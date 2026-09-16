package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.stake.cases.GetDelegation
import com.gemwallet.android.application.stake.cases.GetDelegations
import com.gemwallet.android.application.stake.cases.GetStakeValidator
import com.gemwallet.android.data.coordinators.stake.GetDelegationImpl
import com.gemwallet.android.data.coordinators.stake.GetDelegationsImpl
import com.gemwallet.android.data.coordinators.stake.GetStakeValidatorImpl
import com.gemwallet.android.application.stake.cases.GetValidators
import com.gemwallet.android.data.coordinators.stake.GetValidatorsImpl
import com.gemwallet.android.data.services.gemstone.di.IoDispatcher
import kotlinx.coroutines.CoroutineDispatcher
import com.gemwallet.android.data.services.gemstone.stores.GemstoneStakeStore
import uniffi.gemstone.GemStakeServiceInterface
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object StakeModule {

    @Provides
    @Singleton
    fun provideGetDelegation(stakeStore: GemstoneStakeStore): GetDelegation = GetDelegationImpl(stakeStore)

    @Provides
    @Singleton
    fun provideGetDelegations(
        stakeStore: GemstoneStakeStore,
        stakeService: GemStakeServiceInterface,
        @IoDispatcher ioDispatcher: CoroutineDispatcher,
    ): GetDelegations = GetDelegationsImpl(stakeStore, stakeService, ioDispatcher)

    @Provides
    @Singleton
    fun provideGetValidators(
        stakeStore: GemstoneStakeStore,
        stakeService: GemStakeServiceInterface,
        @IoDispatcher ioDispatcher: CoroutineDispatcher,
    ): GetValidators = GetValidatorsImpl(stakeStore, stakeService, ioDispatcher)

    @Provides
    @Singleton
    fun provideGetStakeValidator(stakeStore: GemstoneStakeStore): GetStakeValidator = GetStakeValidatorImpl(stakeStore)

}
