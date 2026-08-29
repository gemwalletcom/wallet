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
import com.gemwallet.android.cases.stake.SyncStakeDelegations
import com.gemwallet.android.data.coordinators.stake.GetRecommendedValidatorIdsImpl
import com.gemwallet.android.data.coordinators.stake.GetValidatorsImpl
import com.gemwallet.android.data.coordinators.stake.SyncStakeDelegationsImpl
import com.gemwallet.android.data.service.store.database.StakeDao
import uniffi.gemstone.GemStakeRulesService
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
    fun provideGetDelegation(stakeDao: StakeDao): GetDelegation = GetDelegationImpl(stakeDao)

    @Provides
    @Singleton
    fun provideGetDelegations(stakeDao: StakeDao): GetDelegations = GetDelegationsImpl(stakeDao)

    @Provides
    @Singleton
    fun provideGetValidators(stakeDao: StakeDao, stakeRules: GemStakeRulesService): GetValidators = GetValidatorsImpl(stakeDao, stakeRules)

    @Provides
    @Singleton
    fun provideGetRecommendedValidatorIds(stakeRules: GemStakeRulesService): GetRecommendedValidatorIds = GetRecommendedValidatorIdsImpl(stakeRules)

    @Provides
    @Singleton
    fun provideGetRecommendedValidator(getValidators: GetValidators, stakeRules: GemStakeRulesService): GetRecommendedValidator =
        GetRecommendedValidatorImpl(getValidators, stakeRules)

    @Provides
    @Singleton
    fun provideGetStakeValidator(stakeDao: StakeDao): GetStakeValidator = GetStakeValidatorImpl(stakeDao)

    @Provides
    @Singleton
    fun provideSyncStakeDelegations(stakeService: GemStakeService): SyncStakeDelegations = SyncStakeDelegationsImpl(stakeService)
}
