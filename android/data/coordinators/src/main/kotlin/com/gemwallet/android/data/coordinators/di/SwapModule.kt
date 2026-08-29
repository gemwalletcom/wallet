package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.application.swap.cases.BuildSwapConfirmParams
import com.gemwallet.android.application.swap.cases.RequestSwapQuotes
import com.gemwallet.android.application.swap.cases.SearchSwapAssets
import com.gemwallet.android.data.coordinators.swap.BuildSwapConfirmParamsImpl
import com.gemwallet.android.data.coordinators.swap.RequestSwapQuotesImpl
import com.gemwallet.android.data.coordinators.swap.SearchSwapAssetsImpl
import com.gemwallet.android.data.adapters.assets.AssetsSearchService
import com.gemwallet.android.data.adapters.gemstone.GemstoneKeystorePassword
import com.gemwallet.android.application.session.cases.GetSession
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton
import uniffi.gemstone.AlienProvider
import uniffi.gemstone.GemKeystore
import com.gemwallet.android.data.adapters.gemstone.GemstoneSwapStore
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
        getSession: GetSession,
        swapService: GemSwapServiceInterface,
    ): RequestSwapQuotes = RequestSwapQuotesImpl(
        getSession = getSession,
        swapService = swapService,
    )

    @Singleton
    @Provides
    fun provideBuildSwapConfirmParams(
        getSession: GetSession,
        swapService: GemSwapServiceInterface,
    ): BuildSwapConfirmParams = BuildSwapConfirmParamsImpl(
        getSession = getSession,
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
