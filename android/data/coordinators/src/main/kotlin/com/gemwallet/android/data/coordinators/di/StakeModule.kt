package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.stake.cases.GetDelegation
import com.gemwallet.android.application.stake.cases.GetDelegations
import com.gemwallet.android.application.stake.cases.GetRecommendedValidator
import com.gemwallet.android.application.stake.cases.GetStakeValidator
import com.gemwallet.android.data.coordinators.stake.GetDelegationImpl
import com.gemwallet.android.data.coordinators.stake.GetDelegationsImpl
import com.gemwallet.android.data.coordinators.stake.GetRecommendedValidatorImpl
import com.gemwallet.android.data.coordinators.stake.GetStakeValidatorImpl
import com.gemwallet.android.application.stake.cases.GetRecommendedValidatorIds
import com.gemwallet.android.application.stake.cases.GetValidators
import com.gemwallet.android.application.stake.cases.SyncStakeDelegations
import com.gemwallet.android.data.coordinators.stake.GetRecommendedValidatorIdsImpl
import com.gemwallet.android.data.coordinators.stake.GetValidatorsImpl
import com.gemwallet.android.data.coordinators.stake.SyncStakeDelegationsImpl
import com.gemwallet.android.data.repositories.gemstone.GemstoneStakeStore
import uniffi.gemstone.GemStakeConfigService
import uniffi.gemstone.GemStakeService
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
    fun provideGemStakeConfigService(): GemStakeConfigService = GemStakeConfigService()

    @Provides
    @Singleton
    fun provideGetDelegation(stakeStore: GemstoneStakeStore): GetDelegation = GetDelegationImpl(stakeStore)

    @Provides
    @Singleton
    fun provideGetDelegations(stakeStore: GemstoneStakeStore): GetDelegations = GetDelegationsImpl(stakeStore)

    @Provides
    @Singleton
    fun provideGetValidators(stakeStore: GemstoneStakeStore, stakeConfig: GemStakeConfigService): GetValidators = GetValidatorsImpl(stakeStore, stakeConfig)

    @Provides
    @Singleton
    fun provideGetRecommendedValidatorIds(stakeConfig: GemStakeConfigService): GetRecommendedValidatorIds = GetRecommendedValidatorIdsImpl(stakeConfig)

    @Provides
    @Singleton
    fun provideGetRecommendedValidator(getValidators: GetValidators, stakeConfig: GemStakeConfigService): GetRecommendedValidator =
        GetRecommendedValidatorImpl(getValidators, stakeConfig)

    @Provides
    @Singleton
    fun provideGetStakeValidator(stakeStore: GemstoneStakeStore): GetStakeValidator = GetStakeValidatorImpl(stakeStore)

    @Provides
    @Singleton
    fun provideSyncStakeDelegations(stakeService: GemStakeService): SyncStakeDelegations = SyncStakeDelegationsImpl(stakeService)
}
