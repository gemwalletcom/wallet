package com.gemwallet.android.features.banner.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.banner.cases.GetActiveBanners
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toGemKey
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Banner
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemBannerAction
import uniffi.gemstone.GemBannerContent
import uniffi.gemstone.GemBannerServiceInterface
import javax.inject.Inject

data class BannerUIModel(
    val banner: Banner,
    val content: GemBannerContent,
)

@HiltViewModel
class BannersViewModel @Inject constructor(
    private val getActiveBanners: GetActiveBanners,
    private val service: GemBannerServiceInterface,
) : ViewModel() {

    val banners = MutableStateFlow<List<BannerUIModel>>(emptyList())
    private var scene: Pair<Asset?, Boolean> = null to true

    fun init(asset: Asset?, isGlobal: Boolean) {
        scene = asset to isGlobal
        viewModelScope.launch(Dispatchers.IO) {
            val items = getActiveBanners(asset, isGlobal).map { BannerUIModel(it, service.bannerContent(it.event.toJson(), it.asset?.toGem())) }
            banners.update { items }
        }
    }

    fun onSelect(banner: Banner) = apply(banner, GemBannerAction.Event(banner.event.toJson()))

    fun onCancel(banner: Banner) = apply(banner, GemBannerAction.Close)

    private fun apply(banner: Banner, action: GemBannerAction) = viewModelScope.launch(Dispatchers.IO) {
        service.applyAction(banner.toGemKey(), action)
        init(scene.first, scene.second)
    }
}
