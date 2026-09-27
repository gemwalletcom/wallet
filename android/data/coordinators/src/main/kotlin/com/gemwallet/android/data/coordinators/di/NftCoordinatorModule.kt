package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.nft.cases.GetNftAssetDetails
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.coordinators.nft.GetNftAssetDetailsImpl
import com.gemwallet.android.data.services.store.queries.NFTAssetQuery
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemNftServiceInterface
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object NftCoordinatorModule {

    @Provides
    @Singleton
    fun provideGetNftAssetDetails(getSession: GetSession, nftAssetQuery: NFTAssetQuery, nftService: GemNftServiceInterface): GetNftAssetDetails = GetNftAssetDetailsImpl(getSession, nftAssetQuery, nftService)
}
