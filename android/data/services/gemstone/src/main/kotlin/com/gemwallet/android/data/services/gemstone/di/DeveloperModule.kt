package com.gemwallet.android.data.services.gemstone.di

import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.BannersDao
import com.gemwallet.android.data.service.store.database.PricesDao
import com.gemwallet.android.data.service.store.database.StakeDao
import com.gemwallet.android.data.service.store.database.TransactionsDao
import com.gemwallet.android.data.services.gemstone.device.GemstoneDevicePlatform
import com.gemwallet.android.data.services.gemstone.stores.GemstoneDeveloperStore
import com.gemwallet.android.data.services.gemstone.stores.GemstoneTransactionStateStore
import com.gemwallet.android.data.services.gemstone.stores.GemstoneTransactionStore
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemDeveloperService
import uniffi.gemstone.GemDeveloperServiceInterface
import uniffi.gemstone.GemPerpetualService
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemWalletPreferencesService

@InstallIn(SingletonComponent::class)
@Module
object DeveloperModule {

    @Provides
    fun provideGemstoneDeveloperStore(
        transactionsDao: TransactionsDao,
        assetsDao: AssetsDao,
        stakeDao: StakeDao,
        bannersDao: BannersDao,
        pricesDao: PricesDao,
    ): GemstoneDeveloperStore = GemstoneDeveloperStore(transactionsDao, assetsDao, stakeDao, bannersDao, pricesDao)

    @Provides
    fun provideGemDeveloperService(
        platform: GemstoneDevicePlatform,
        preferencesService: GemPreferencesService,
        walletPreferencesService: GemWalletPreferencesService,
        transactionStateStore: GemstoneTransactionStateStore,
        transactionStore: GemstoneTransactionStore,
        perpetualService: GemPerpetualService,
        developerStore: GemstoneDeveloperStore,
    ): GemDeveloperServiceInterface = GemDeveloperService(
        platform,
        preferencesService,
        walletPreferencesService,
        transactionStateStore,
        transactionStore,
        perpetualService,
        developerStore,
    )
}
