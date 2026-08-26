package com.gemwallet.android.data.services.gemapi

import com.wallet.core.primitives.AssetBasic
import com.wallet.core.primitives.AssetFull
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.FiatAssets
import com.wallet.core.primitives.SearchResponse
import retrofit2.http.Body
import retrofit2.http.GET
import retrofit2.http.POST
import retrofit2.http.Path
import retrofit2.http.Query

interface GemApiClient {

    @GET("/v1/assets/{asset_id}")
    suspend fun getAsset(@Path("asset_id") assetId: String): AssetFull

    @POST("/v1/assets")
    suspend fun getAssets(
        @Body ids: List<AssetId>,
    ): List<AssetBasic>

    @GET("/v1/fiat/assets/{type}")
    suspend fun getFiatAssets(@Path("type") type: String): FiatAssets

    @GET("/v1/swap/assets")
    suspend fun getSwapAssets(): FiatAssets

    @GET("/v1/assets/search")
    suspend fun searchAssets(
        @Query("query") query: String,
        @Query("chains") chains: String,
    ): List<AssetBasic>

    @GET("/v1/search")
    suspend fun search(
        @Query("query") query: String,
        @Query("chains") chains: String,
        @Query("tags") tags: String,
    ): SearchResponse
}
