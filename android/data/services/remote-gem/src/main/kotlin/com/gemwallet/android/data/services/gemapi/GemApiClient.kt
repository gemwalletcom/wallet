package com.gemwallet.android.data.services.gemapi

import com.wallet.core.primitives.AssetBasic
import com.wallet.core.primitives.AssetFull
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Charts
import com.wallet.core.primitives.ConfigResponse
import retrofit2.http.Body
import retrofit2.http.GET
import retrofit2.http.POST
import retrofit2.http.Path
import retrofit2.http.Query

interface GemApiClient {
    @GET("/v1/config")
    suspend fun getConfig(): ConfigResponse

    @GET("/v1/charts/{asset_id}")
    suspend fun getChart(@Path("asset_id") assetId: String, @Query("period") period: String): Charts

    @GET("/v1/assets/{asset_id}")
    suspend fun getAsset(@Path("asset_id") assetId: String): AssetFull

    @POST("/v1/assets")
    suspend fun getAssets(
        @Body ids: List<AssetId>,
    ): List<AssetBasic>

    @GET("/v1/assets/search")
    suspend fun search(
        @Query("query") query: String,
        @Query("chains") chains: String,
        @Query("tags") tags: String,
    ): List<AssetBasic>
}
