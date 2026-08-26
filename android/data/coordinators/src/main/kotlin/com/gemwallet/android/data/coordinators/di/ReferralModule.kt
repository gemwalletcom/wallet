package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.assets.coordinators.EnableAsset
import com.gemwallet.android.application.referral.coordinators.CreateReferral
import com.gemwallet.android.application.referral.coordinators.GetRewards
import com.gemwallet.android.application.referral.coordinators.Redeem
import com.gemwallet.android.application.referral.coordinators.UseReferralCode
import com.gemwallet.android.data.coordinators.referral.CreateReferralImpl
import com.gemwallet.android.data.coordinators.referral.GetRewardsImpl
import com.gemwallet.android.data.coordinators.referral.RedeemImpl
import com.gemwallet.android.data.coordinators.referral.UseReferralCodeImpl
import com.gemwallet.android.data.repositories.session.SessionRepository
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemRewardsService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object ReferralModule {
    @Provides
    @Singleton
    fun provideCreateReferral(
        rewardsService: GemRewardsService,
    ): CreateReferral {
        return CreateReferralImpl(
            rewardsService = rewardsService,
        )
    }

    @Provides
    @Singleton
    fun provideGetRewards(
        rewardsService: GemRewardsService,
    ): GetRewards {
        return GetRewardsImpl(
            rewardsService = rewardsService,
        )
    }

    @Provides
    @Singleton
    fun provideRedeem(
        sessionRepository: SessionRepository,
        rewardsService: GemRewardsService,
        enableAsset: EnableAsset,
    ): Redeem {
        return RedeemImpl(
            sessionRepository = sessionRepository,
            rewardsService = rewardsService,
            enableAsset = enableAsset,
        )
    }

    @Provides
    @Singleton
    fun provideUseReferralCode(
        rewardsService: GemRewardsService,
    ): UseReferralCode {
        return UseReferralCodeImpl(
            rewardsService = rewardsService,
        )
    }
}