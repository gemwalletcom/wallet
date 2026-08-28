package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.application.swap.coordinators.BuildSwapConfirmParams
import com.gemwallet.android.application.swap.coordinators.RequestSwapQuotes
import com.gemwallet.android.application.swap.coordinators.SearchSwapAssets
import com.gemwallet.android.data.coordinators.swap.BuildSwapConfirmParamsImpl
import com.gemwallet.android.data.coordinators.swap.RequestSwapQuotesImpl
import com.gemwallet.android.data.coordinators.swap.SearchSwapAssetsImpl
import com.gemwallet.android.data.repositories.assets.AssetsSearchService
import com.gemwallet.android.data.repositories.gemstone.GemstoneKeystorePassword
import com.gemwallet.android.data.repositories.session.SessionRepository
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton
import uniffi.gemstone.AlienProvider
import uniffi.gemstone.GemKeystore
import com.gemwallet.android.data.repositories.gemstone.GemstoneSwapStore
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.TransactionsDao
import uniffi.gemstone.GemSwapService
import uniffi.gemstone.GemSwapServiceInterface
import uniffi.gemstone.GemSwapper

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
    fun provideGemSwapService(
        gemSwapper: GemSwapper,
        gemKeystore: GemKeystore,
        passwordStore: PasswordStore,
        assetsDao: AssetsDao,
        transactionsDao: TransactionsDao,
    ): GemSwapServiceInterface = GemSwapService(
        swapper = gemSwapper,
        keystore = gemKeystore,
        password = GemstoneKeystorePassword(passwordStore),
        store = GemstoneSwapStore(assetsDao, transactionsDao),
    )

    @Singleton
    @Provides
    fun provideRequestSwapQuotes(
        sessionRepository: SessionRepository,
        swapService: GemSwapServiceInterface,
    ): RequestSwapQuotes = RequestSwapQuotesImpl(
        sessionRepository = sessionRepository,
        swapService = swapService,
    )

    @Singleton
    @Provides
    fun provideBuildSwapConfirmParams(
        sessionRepository: SessionRepository,
        swapService: GemSwapServiceInterface,
    ): BuildSwapConfirmParams = BuildSwapConfirmParamsImpl(
        sessionRepository = sessionRepository,
        swapService = swapService,
    )

    @Singleton
    @Provides
    fun provideSearchSwapAssets(
        searchService: AssetsSearchService,
        swapService: GemSwapServiceInterface,
    ): SearchSwapAssets = SearchSwapAssetsImpl(
        searchService = searchService,
        swapService = swapService,
    )


}
