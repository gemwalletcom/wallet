package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.stake.coordinators.GetDelegation
import com.gemwallet.android.application.stake.coordinators.GetDelegations
import com.gemwallet.android.application.stake.coordinators.GetRecommendedValidator
import com.gemwallet.android.application.stake.coordinators.GetStakeValidator
import com.gemwallet.android.data.coordinators.stake.GetDelegationImpl
import com.gemwallet.android.data.coordinators.stake.GetDelegationsImpl
import com.gemwallet.android.data.coordinators.stake.GetRecommendedValidatorImpl
import com.gemwallet.android.data.coordinators.stake.GetStakeValidatorImpl
import com.gemwallet.android.data.repositories.stake.StakeRepository
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
    fun provideGetDelegation(stakeRepository: StakeRepository): GetDelegation =
        GetDelegationImpl(stakeRepository)

    @Provides
    @Singleton
    fun provideGetDelegations(stakeRepository: StakeRepository): GetDelegations =
        GetDelegationsImpl(stakeRepository)

    @Provides
    @Singleton
    fun provideGetRecommendedValidator(stakeRepository: StakeRepository): GetRecommendedValidator =
        GetRecommendedValidatorImpl(stakeRepository)

    @Provides
    @Singleton
    fun provideGetStakeValidator(stakeRepository: StakeRepository): GetStakeValidator =
        GetStakeValidatorImpl(stakeRepository)
}
