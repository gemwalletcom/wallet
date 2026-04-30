package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.GetAuthPayload
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
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object ReferralModule {
    @Provides
    @Singleton
    fun provideCreateReferral(
        gemDeviceApiClient: GemDeviceApiClient,
        getAuthPayload: GetAuthPayload,
    ): CreateReferral {
        return CreateReferralImpl(
            gemDeviceApiClient = gemDeviceApiClient,
            getAuthPayload = getAuthPayload
        )
    }

    @Provides
    @Singleton
    fun provideGetRewards(
        gemDeviceApiClient: GemDeviceApiClient,
    ): GetRewards {
        return GetRewardsImpl(
            gemDeviceApiClient = gemDeviceApiClient,
        )
    }

    @Provides
    @Singleton
    fun provideRedeem(
        sessionRepository: SessionRepository,
        gemDeviceApiClient: GemDeviceApiClient,
        getAuthPayload: GetAuthPayload,
        enableAsset: EnableAsset,
    ): Redeem {
        return RedeemImpl(
            sessionRepository = sessionRepository,
            gemDeviceApiClient = gemDeviceApiClient,
            getAuthPayload = getAuthPayload,
            enableAsset = enableAsset,
        )
    }

    @Provides
    @Singleton
    fun provideUseReferralCode(
        gemDeviceApiClient: GemDeviceApiClient,
        getAuthPayload: GetAuthPayload
    ): UseReferralCode {
        return UseReferralCodeImpl(
            gemDeviceApiClient = gemDeviceApiClient,
            getAuthPayload = getAuthPayload
        )
    }
}