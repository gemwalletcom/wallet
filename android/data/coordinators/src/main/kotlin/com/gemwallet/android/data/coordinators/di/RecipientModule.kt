package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.recipient.coordinators.GetRecipientAssetInfo
import com.gemwallet.android.application.recipient.coordinators.GetWallets
import com.gemwallet.android.data.coordinators.recipient.GetRecipientAssetInfoImpl
import com.gemwallet.android.data.coordinators.recipient.GetWalletsImpl
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object RecipientModule {

    @Provides
    @Singleton
    fun provideGetWallets(
        walletsRepository: WalletsRepository,
    ): GetWallets = GetWalletsImpl(walletsRepository)

    @Provides
    @Singleton
    fun provideGetRecipientAssetInfo(
        assetsRepository: AssetsRepository,
    ): GetRecipientAssetInfo = GetRecipientAssetInfoImpl(assetsRepository)
}
