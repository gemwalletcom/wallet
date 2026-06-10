package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.application.swap.coordinators.BuildSwapConfirmParams
import com.gemwallet.android.application.swap.coordinators.GetSwapQuoteData
import com.gemwallet.android.application.swap.coordinators.GetSwapQuotes
import com.gemwallet.android.application.swap.coordinators.GetSwapSupported
import com.gemwallet.android.application.swap.coordinators.RequestSwapQuotes
import com.gemwallet.android.application.swap.coordinators.SearchSwapAssets
import com.gemwallet.android.blockchain.services.GemSignMessageOperator
import com.gemwallet.android.data.coordinators.swap.BuildSwapConfirmParamsImpl
import com.gemwallet.android.data.coordinators.swap.GetSwapQuoteDataImpl
import com.gemwallet.android.data.coordinators.swap.GetSwapQuotesImpl
import com.gemwallet.android.data.coordinators.swap.GetSwapSupportedImpl
import com.gemwallet.android.data.coordinators.swap.RequestSwapQuotesImpl
import com.gemwallet.android.data.coordinators.swap.SearchSwapAssetsImpl
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.AlienProvider
import uniffi.gemstone.GemSwapper
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object SwapModule {

    @Singleton
    @Provides
    fun provideGemSwapper(
        alienProvider: AlienProvider,
    ): GemSwapper = GemSwapper(alienProvider)

    @Singleton
    @Provides
    fun provideGetSwapQuotes(
        gemSwapper: GemSwapper,
    ): GetSwapQuotes = GetSwapQuotesImpl(gemSwapper)

    @Singleton
    @Provides
    fun provideGetSwapSupported(
        gemSwapper: GemSwapper,
    ): GetSwapSupported = GetSwapSupportedImpl(gemSwapper)

    @Singleton
    @Provides
    fun provideGetSwapQuoteData(
        gemSwapper: GemSwapper,
        passwordStore: PasswordStore,
        signMessageOperator: GemSignMessageOperator,
    ): GetSwapQuoteData = GetSwapQuoteDataImpl(
        gemSwapper = gemSwapper,
        passwordStore = passwordStore,
        signMessageOperator = signMessageOperator,
    )

    @Singleton
    @Provides
    fun provideRequestSwapQuotes(
        getSwapQuotes: GetSwapQuotes,
    ): RequestSwapQuotes = RequestSwapQuotesImpl(getSwapQuotes)

    @Singleton
    @Provides
    fun provideBuildSwapConfirmParams(
        sessionRepository: SessionRepository,
        getSwapQuoteData: GetSwapQuoteData,
    ): BuildSwapConfirmParams = BuildSwapConfirmParamsImpl(
        sessionRepository = sessionRepository,
        getSwapQuoteData = getSwapQuoteData,
    )

    @Singleton
    @Provides
    fun provideSearchSwapAssets(
        assetsRepository: AssetsRepository,
        getSwapSupported: GetSwapSupported,
    ): SearchSwapAssets = SearchSwapAssetsImpl(
        assetsRepository = assetsRepository,
        getSwapSupported = getSwapSupported,
    )
}
