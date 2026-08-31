package com.gemwallet.android.features.banner.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.banner.cases.ApplyBannerAction
import com.gemwallet.android.application.banner.cases.GetActiveBanners
import com.gemwallet.android.application.banner.cases.GetBannerContent
import com.gemwallet.android.domains.banner.BannerAction
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Banner
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemBannerContent
import javax.inject.Inject

data class BannerUIModel(
    val banner: Banner,
    val content: GemBannerContent,
)

@HiltViewModel
class BannersViewModel @Inject constructor(
    private val getActiveBanners: GetActiveBanners,
    private val getBannerContent: GetBannerContent,
    private val applyBannerAction: ApplyBannerAction,
) : ViewModel() {

    val banners = MutableStateFlow<List<BannerUIModel>>(emptyList())
    private var scene: Pair<Asset?, Boolean> = null to true

    fun init(asset: Asset?, isGlobal: Boolean) {
        scene = asset to isGlobal
        viewModelScope.launch(Dispatchers.IO) {
            val items = getActiveBanners(asset, isGlobal).map { BannerUIModel(it, getBannerContent(it)) }
            banners.update { items }
        }
    }

    fun onSelect(banner: Banner) = apply(banner, BannerAction.Event(banner.event))

    fun onCancel(banner: Banner) = apply(banner, BannerAction.Close)

    private fun apply(banner: Banner, action: BannerAction) = viewModelScope.launch {
        applyBannerAction(banner, action)
        init(scene.first, scene.second)
    }
}
