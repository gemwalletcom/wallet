package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.assets.cases.EnableAsset
import com.gemwallet.android.application.referral.cases.CreateReferral
import com.gemwallet.android.application.referral.cases.GetRewards
import com.gemwallet.android.application.referral.cases.Redeem
import com.gemwallet.android.application.referral.cases.UseReferralCode
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
        rewardsService: GemRewardsService,
    ): Redeem {
        return RedeemImpl(rewardsService = rewardsService,
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